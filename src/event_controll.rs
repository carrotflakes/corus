use std::{collections::VecDeque, marker::PhantomData};

use crate::{node::Node, proc_context::ProcContext};

pub trait Event<T: 'static> {
    type Node: Node<T>;

    fn get_time(&self) -> f64;
    fn dispatch(&self, node: &mut Self::Node);
}

pub struct EventControll<T: 'static, E: Event<T>> {
    node: E::Node,
    events: VecDeque<E>,
    _t: PhantomData<T>,
}

impl<T: 'static, E: Event<T>> EventControll<T, E> {
    pub fn new(node: E::Node) -> Self {
        Self {
            node,
            events: Vec::new().into(),
            _t: Default::default(),
        }
    }

    pub fn push_event(&mut self, event: E) {
        let time = event.get_time();
        for (i, e) in self.events.iter().enumerate() {
            if time < e.get_time() {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push_back(event);
    }
}

impl<T: 'static, E: Event<T>> Node<T> for EventControll<T, E> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        while let Some(event) = self.events.front_mut() {
            if ctx.time < event.get_time() {
                break;
            }
            event.dispatch(&mut self.node);
            self.events.pop_front();
        }
        self.node.proc(ctx)
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<T: 'static, E: Event<T>> AsMut<Self> for EventControll<T, E> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
