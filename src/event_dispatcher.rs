use std::{cell::RefCell, collections::VecDeque, marker::PhantomData, rc::Rc};

use crate::{core::Node, proc_context::ProcContext};

pub use crate::contrib::event_control::Event;

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

    pub fn wrap<E: Event>(&mut self, t: E::Target) -> EventControl<E> {
        EventControl::new(self.events.clone(), Box::new(t))
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

    pub fn finish<T: 'static, N: Node<T>>(self, node: N) -> EventDispatchNode<T, N> {
        EventDispatchNode {
            node,
            event_queue: self,
            _t: Default::default(),
        }
    }
}

pub struct EventControl<E: Event> {
    events: Rc<RefCell<VecDeque<(f64, Box<dyn FnMut(f64)>)>>>,
    target: Box<E::Target>,
    _t: PhantomData<E>,
}

impl<E: Event> EventControl<E> {
    fn new(
        events: Rc<RefCell<VecDeque<(f64, Box<dyn FnMut(f64)>)>>>,
        target: Box<E::Target>,
    ) -> Self {
        Self {
            events,
            target,
            _t: Default::default(),
        }
    }

    pub fn push(&mut self, time: f64, event: E) {
        let target =
            unsafe { std::mem::transmute::<_, &'static mut E::Target>(self.target.as_mut()) };
        // let event = Box::new((event, target));
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

impl<T: 'static, N: Node<T>, E: Event<Target = N>> Node<T> for EventControl<E> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.target.proc(ctx)
    }

    fn lock(&mut self) {
        self.target.lock();
    }

    fn unlock(&mut self) {
        self.target.unlock();
    }
}


pub struct EventDispatchNode<T: 'static, N: Node<T>> {
    node: N,
    event_queue: EventQueue,
    _t: PhantomData<T>,
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
