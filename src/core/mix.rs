use std::ops::Add;

use super::{Node, ProcContext};

pub struct Mix<A>
where
    A: Node,
    A::Output: Clone + 'static + Add<Output = A::Output> + Default,
{
    nodes: Vec<A>,
}

impl<A> Mix<A>
where
    A: Node,
    A::Output: Clone + 'static + Add<Output = A::Output> + Default,
{
    pub fn new(nodes: Vec<A>) -> Self {
        Mix { nodes }
    }
}

impl<A> Node for Mix<A>
where
    A: Node,
    A::Output: Clone + 'static + Add<Output = A::Output> + Default,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let mut v = Default::default();
        for node in self.nodes.iter_mut() {
            v = v + node.proc(ctx);
        }
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        for node in &mut self.nodes {
            node.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        for node in &mut self.nodes {
            node.unlock();
        }
    }
}
