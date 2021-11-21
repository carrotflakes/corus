use crate::{signal::Signal, Node, ProcContext};

pub struct FirstOrderFilter<S, N, A>
where
    S: Signal + Default,
    N: Node<Output = S>,
    A: Node<Output = S>,
{
    node: N,
    k: A,
    prev: N::Output,
}

impl<S, N, A> FirstOrderFilter<S, N, A>
where
    S: Signal + Default,
    N: Node<Output = S>,
    A: Node<Output = S>,
{
    pub fn new(node: N, k: A) -> Self {
        Self {
            node,
            k,
            prev: N::Output::default(),
        }
    }
}

impl<S, N, A> Node for FirstOrderFilter<S, N, A>
where
    S: Signal + Default,
    N: Node<Output = S>,
    A: Node<Output = S>,
{
    type Output = N::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let k = self.k.proc(ctx);
        let x = self.node.proc(ctx);
        self.prev = self.prev.clone() + k * (x - self.prev.clone());
        self.prev.clone()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.k.lock(ctx);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.k.unlock();
        self.node.unlock();
    }
}
