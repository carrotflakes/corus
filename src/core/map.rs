use super::{Node, ProcContext};

pub struct Map<O, F, A>
where
    O: Clone + 'static,
    F: Fn(A::Output) -> O,
    A: Node,
    A::Output: Clone,
{
    node: A,
    f: F,
}

impl<O, F, A> Map<O, F, A>
where
O: Clone + 'static,
F: Fn(A::Output) -> O,
A: Node,
A::Output: Clone,
{
    pub fn new(node: A, f: F) -> Self {
        Map {
            node,
            f,
        }
    }
}

impl<O, F, A> Node for Map<O, F, A>
where
O: Clone + 'static,
F: Fn(A::Output) -> O,
A: Node,
A::Output: Clone,
{
    type Output = O;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        (self.f)(self.node.proc(ctx))
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
