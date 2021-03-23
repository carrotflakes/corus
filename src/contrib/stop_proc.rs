use crate::Node;

pub struct StopProc<A>
where
    A: Node,
    A::Output: Default,
{
    node: A,
    pub enabled: bool,
}

impl<A> StopProc<A>
where
    A: Node,
    A::Output: Default,
{
    pub fn new(node: A, enabled: bool) -> Self {
        Self { node, enabled }
    }
}

impl<A> Node for StopProc<A>
where
    A: Node,
    A::Output: Default,
{
    type Output = A::Output;

    fn proc(&mut self, ctx: &crate::ProcContext) -> Self::Output {
        if self.enabled {
            self.node.proc(ctx)
        } else {
            Default::default()
        }
    }

    fn lock(&mut self, ctx: &crate::ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
