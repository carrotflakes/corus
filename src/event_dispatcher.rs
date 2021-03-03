use std::{
    cell::RefCell,
    collections::VecDeque,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Weak},
};

use crate::{core::Node, proc_context::ProcContext};

pub trait Event: 'static {
    type Target: 'static;

    fn dispatch(&self, time: f64, target: &mut Self::Target);
}

fn caller<E: Event>(event: E, target: &'static mut E::Target) -> Box<dyn FnMut(f64)> {
    Box::new(move |time: f64| event.dispatch(time, target))
}

pub struct EventQueue {
    events: Rc<RefCell<VecDeque<(f64, Box<dyn FnMut(f64)>)>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            events: Rc::new(RefCell::new(Vec::new().into())),
        }
    }

    pub fn get_controller<E: Event>(&mut self, arc: &Arc<E::Target>) -> EventControl<E> {
        EventControl::new(self.events.clone(), Arc::downgrade(arc))
    }

    pub fn dispatch(&mut self, ctx: &ProcContext) {
        let mut events = self.events.borrow_mut();
        while let Some(e) = events.front_mut() {
            if ctx.time < e.0 {
                break;
            }
            (e.1)(e.0);
            events.pop_front();
        }
    }

    pub fn dispatch_node<T: 'static, N: Node<T>>(&self, node: N) -> EventDispatchNode<T, N> {
        EventDispatchNode::new(node, self.clone())
    }
}

impl Clone for EventQueue {
    fn clone(&self) -> Self {
        EventQueue {
            events: self.events.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EventControl<E: Event> {
    events: Rc<RefCell<VecDeque<(f64, Box<dyn FnMut(f64)>)>>>,
    target: Weak<E::Target>,
    _t: PhantomData<E>,
}

impl<E: Event> EventControl<E> {
    fn new(
        events: Rc<RefCell<VecDeque<(f64, Box<dyn FnMut(f64)>)>>>,
        target: Weak<E::Target>,
    ) -> Self {
        Self {
            events,
            target,
            _t: Default::default(),
        }
    }

    pub fn push(&mut self, time: f64, event: E) {
        let target =
            unsafe { std::mem::transmute::<_, &'static mut E::Target>(self.target.as_ptr()) };
        let mut events = self.events.borrow_mut();
        for (i, e) in events.iter().enumerate() {
            if time < e.0 {
                events.insert(i, (time, caller(event, target)));
                return;
            }
        }
        events.push_back((time, caller(event, target)));
    }
}

pub struct EventDispatchNode<T: 'static, N: Node<T>> {
    node: N,
    event_queue: EventQueue,
    _t: PhantomData<T>,
}

impl<T: 'static, N: Node<T>> EventDispatchNode<T, N> {
    pub fn new(node: N, event_queue: EventQueue) -> Self {
        Self {
            node,
            event_queue,
            _t: Default::default(),
        }
    }
}

impl<T: 'static, N: Node<T>> Node<T> for EventDispatchNode<T, N> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.event_queue.dispatch(ctx);
        self.node.proc(ctx)
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

pub struct EventControlInplace<E: Event> {
    target: E::Target,
    events: VecDeque<(f64, E)>,
}

impl<E: Event> EventControlInplace<E> {
    pub fn new(target: E::Target) -> Self {
        Self {
            target,
            events: Vec::new().into(),
        }
    }

    pub fn push_event(&mut self, time: f64, event: E) {
        for (i, e) in self.events.iter().enumerate() {
            if time < e.0 {
                self.events.insert(i, (time, event));
                return;
            }
        }
        self.events.push_back((time, event));
    }
}

impl<T: 'static, N: Node<T>, E: Event<Target = N>> Node<T> for EventControlInplace<E> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        while let Some(e) = self.events.front_mut() {
            if ctx.time < e.0 {
                break;
            }
            e.1.dispatch(e.0, &mut self.target);
            self.events.pop_front();
        }
        self.target.proc(ctx)
    }

    fn lock(&mut self) {
        self.target.lock();
    }

    fn unlock(&mut self) {
        self.target.unlock();
    }
}
