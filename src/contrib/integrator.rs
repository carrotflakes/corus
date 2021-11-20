use crate::{EventListener, signal::Signal};

use super::{Node, ProcContext};

/// Non-frequency-aware integrator.
pub struct Integrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    node: A,
    prev: A::Output,
}

impl<A> Integrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    pub fn new(node: A) -> Self {
        Integrator {
            node,
            prev: A::Output::default(),
        }
    }
}

impl<A> Node for Integrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let v = self.node.proc(ctx);
        self.prev = self.prev.clone() + v;
        self.prev.clone()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

pub enum IntegratorEvent<T> {
    SetValue(T),
}

impl<T: Signal, A: 'static + Node<Output = T>> EventListener<IntegratorEvent<T>> for Integrator<A> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &IntegratorEvent<T>) {
        match event {
            IntegratorEvent::SetValue(value) => {
                self.prev = value.clone();
            }
        }
    }
}
