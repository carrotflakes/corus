use crate::{proc_context::ProcContext, signal::{C1f64, Signal}};

use super::Node;

pub struct Smooth<A>
where
A: Node,
A::Output: Signal,
{
    node: A,
    level: A::Output,
    value: A::Output,
}

impl<A> Smooth<A>
where
A: Node,
A::Output: Signal,
{
    pub fn new(node: A, level: A::Output) -> Self {
        Smooth {
            node,
            level,
            value: Default::default(),
        }
    }
}

impl<A> Node for Smooth<A>
where
    A: Node<Output = C1f64>,
{
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let v = self.node.proc(ctx);
        self.value = self.value * self.level + v * (1.0 - self.level);
        self.value
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
