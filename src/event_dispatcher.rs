use std::{
    collections::VecDeque,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{proc_context::ProcContext, Node};

pub trait EventListener<E>: 'static {
    fn apply_event(&mut self, time: f64, event: &E);
}

struct EventTargetPair<E, L>
where
    E: Send + Sync,
    L: EventListener<E> + Send + Sync,
{
    event: E,
    target: Arc<L>,
}

trait EventDispatch: Send + Sync {
    fn dispatch(&mut self, time: f64);
}

impl<E, L> EventDispatch for EventTargetPair<E, L>
where
    E: Send + Sync,
    L: EventListener<E> + Send + Sync,
{
    #[inline]
    fn dispatch(&mut self, time: f64) {
        let target = unsafe { std::mem::transmute::<_, &mut L>(Arc::as_ptr(&self.target)) };
        target.apply_event(time, &self.event);
    }
}

#[derive(Default)]
pub struct EventQueue {
    events: Arc<Mutex<VecDeque<(f64, Box<dyn EventDispatch>)>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new().into())),
        }
    }

    pub fn get_controller<T, E, L>(&mut self, ec: &EventControllable<T, L>) -> EventControl<L>
    where
        E: Send + Sync,
        L: Node<T> + EventListener<E> + Send + Sync,
    {
        EventControl::new(self.events.clone(), ec.inner())
    }

    pub(crate) fn dispatch(&mut self, current_time: f64) {
        let mut events = self.events.lock().unwrap();
        while let Some(e) = events.front_mut() {
            if current_time < e.0 {
                break;
            }
            e.1.dispatch(e.0);
            events.pop_front();
        }
    }

    fn push_event_dispatch(&self, time: f64, event_dispatch: Box<dyn EventDispatch>) {
        let mut events = self.events.lock().unwrap();
        for (i, e) in events.iter().enumerate() {
            if time < e.0 {
                events.insert(i, (time, event_dispatch));
                return;
            }
        }
        events.push_back((time, event_dispatch));
    }

    pub fn push_event<E, L>(&self, time: f64, event: E, target: Arc<L>)
    where
        E: 'static + Send + Sync,
        L: EventListener<E> + Send + Sync,
    {
        self.push_event_dispatch(time, Box::new(EventTargetPair { event, target }));
    }
}

impl Clone for EventQueue {
    #[inline]
    fn clone(&self) -> Self {
        EventQueue {
            events: self.events.clone(),
        }
    }
}

impl PartialEq for EventQueue {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.events, &other.events)
    }
}

pub trait EventPusher<E> {
    fn push_event(&mut self, time: f64, event: E);
}

pub struct EventControllable<T: 'static, A: 'static + Node<T>> {
    node: Arc<A>,
    _t: PhantomData<T>,
}

impl<T: 'static, A: 'static + Node<T>> EventControllable<T, A> {
    pub fn new(node: A) -> Self {
        Self {
            node: Arc::new(node),
            _t: Default::default(),
        }
    }

    #[inline]
    pub fn inner(&self) -> Arc<A> {
        self.node.clone()
    }
}

impl<T, A> Node<T> for EventControllable<T, A>
where
    T: 'static,
    A: 'static + Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        get_mut(&mut self.node).proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        get_mut(&mut self.node).lock(ctx);
    }

    fn unlock(&mut self) {
        get_mut(&mut self.node).unlock();
    }
}

#[inline]
fn get_mut<T, A>(arc: &mut Arc<A>) -> &mut A
where
    T: 'static,
    A: 'static + Node<T>,
{
    unsafe { std::mem::transmute::<_, &mut A>(Arc::as_ptr(arc)) }
}

#[derive(Clone)]
pub struct EventControl<L> {
    events: Arc<Mutex<VecDeque<(f64, Box<dyn EventDispatch>)>>>,
    target: Arc<L>,
}

impl<L> EventControl<L> {
    fn new(events: Arc<Mutex<VecDeque<(f64, Box<dyn EventDispatch>)>>>, target: Arc<L>) -> Self {
        Self { events, target }
    }
}

impl<E, L> EventPusher<E> for EventControl<L>
where
    E: 'static + Send + Sync,
    L: EventListener<E> + Send + Sync,
{
    fn push_event(&mut self, time: f64, event: E) {
        let target = self.target.clone();
        let pair = Box::new(EventTargetPair { event, target });
        let mut events = self.events.lock().unwrap();
        for (i, e) in events.iter().enumerate() {
            if time < e.0 {
                events.insert(i, (time, pair));
                return;
            }
        }
        events.push_back((time, pair));
    }
}

#[derive(Default)]
pub struct EventSchedule<A: 'static> {
    events: EventQueue,
    target: Arc<A>,
}

impl<E: 'static + Sync + Send, A: 'static + EventListener<E> + Sync + Send> EventPusher<E>
    for EventSchedule<A>
{
    fn push_event(&mut self, time: f64, event: E) {
        self.events.push_event(time, event, self.target.clone());
    }
}

impl<A: 'static> Clone for EventSchedule<A> {
    fn clone(&self) -> Self {
        EventSchedule {
            events: self.events.clone(),
            target: self.target.clone(),
        }
    }
}

pub struct EventScheduleNode<T: 'static, A: 'static + Node<T>> {
    target: EventControllable<T, A>,
    schedule: EventSchedule<A>,
}

impl<T: 'static, A: 'static + Node<T>> EventScheduleNode<T, A> {
    pub fn new(target: EventControllable<T, A>) -> Self {
        Self {
            schedule: EventSchedule {
                events: Default::default(),
                target: target.node.clone(),
            },
            target,
        }
    }

    pub fn get_scheduler(&mut self) -> EventSchedule<A> {
        self.schedule.clone()
    }
}

impl<T, A> Node<T> for EventScheduleNode<T, A>
where
    T: 'static,
    A: 'static + Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.target.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        let mut events = self.schedule.events.events.lock().unwrap();
        let time = ctx.current_time + ctx.rest_proc_samples as f64 / ctx.sample_rate as f64;

        while !events.is_empty() {
            if time < events[0].0 {
                break;
            }
            let first = events.pop_front().unwrap();
            ctx.event_queue.push_event_dispatch(first.0, first.1);
        }
        self.target.lock(ctx);
    }

    fn unlock(&mut self) {
        self.target.unlock();
    }
}

pub struct EventControlInplace<E, L: EventListener<E>> {
    target: L,
    events: VecDeque<(f64, E)>,
}

impl<E, L: EventListener<E>> EventControlInplace<E, L> {
    pub fn new(target: L) -> Self {
        Self {
            target,
            events: Vec::new().into(),
        }
    }
}

impl<E, L: EventListener<E>> EventPusher<E> for EventControlInplace<E, L> {
    fn push_event(&mut self, time: f64, event: E) {
        for (i, e) in self.events.iter().enumerate() {
            if time < e.0 {
                self.events.insert(i, (time, event));
                return;
            }
        }
        self.events.push_back((time, event));
    }
}

impl<T: 'static, L: Node<T> + EventListener<E>, E> Node<T> for EventControlInplace<E, L> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        while let Some(e) = self.events.front_mut() {
            if ctx.current_time < e.0 {
                break;
            }
            self.target.apply_event(e.0, &e.1);
            self.events.pop_front();
        }
        self.target.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.target.lock(ctx);
    }

    fn unlock(&mut self) {
        self.target.unlock();
    }
}
