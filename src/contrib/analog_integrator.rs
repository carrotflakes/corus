use crate::{EventListener, signal::Signal};

use super::{Node, ProcContext};

/// Frequency-aware integrator.
pub struct AnalogIntegrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    node: A,
    prev: A::Output,
}

impl<A> AnalogIntegrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    pub fn new(node: A) -> Self {
        AnalogIntegrator {
            node,
            prev: A::Output::default(),
        }
    }
}

impl<A> Node for AnalogIntegrator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let v = self.node.proc(ctx) / ctx.sample_rate as f64;
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

pub enum AnalogIntegratorEvent<T> {
    SetValue(T),
}

impl<T: Signal, A: 'static + Node<Output = T>> EventListener<AnalogIntegratorEvent<T>> for AnalogIntegrator<A> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &AnalogIntegratorEvent<T>) {
        match event {
            AnalogIntegratorEvent::SetValue(value) => {
                self.prev = value.clone();
            }
        }
    }
}
