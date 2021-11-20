use crate::signal::Signal;

use super::{Node, ProcContext};

/// Frequency-aware differentiator.
pub struct AnalogDifferentiator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    node: A,
    prev: A::Output,
}

impl<A> AnalogDifferentiator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    pub fn new(node: A) -> Self {
        AnalogDifferentiator {
            node,
            prev: A::Output::default(),
        }
    }
}

impl<A> Node for AnalogDifferentiator<A>
where
    A: Node,
    A::Output: Signal + Default + std::ops::Sub<Output = A::Output>,
    <A::Output as Signal>::Float: From<f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let v = self.node.proc(ctx) / ctx.sample_rate as f64;
        let prev = self.prev.clone();
        self.prev = v.clone();
        v - prev
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
