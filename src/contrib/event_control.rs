use std::collections::VecDeque;

use crate::{core::Node, proc_context::ProcContext};

pub trait Event: 'static {
    type Target: 'static;

    fn dispatch(&self, time: f64, target: &mut Self::Target);
}

pub struct EventControl<E: Event> {
    target: E::Target,
    events: VecDeque<(f64, E)>,
}

impl<E: Event> EventControl<E> {
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

impl<T: 'static, N: Node<T>, E: Event<Target = N>> Node<T> for EventControl<E> {
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

impl<E: Event> AsMut<Self> for EventControl<E> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
