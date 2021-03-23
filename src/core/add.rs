use super::{Node, ProcContext};

pub struct Add<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Add<Output = A::Output>,
{
    a: A,
    b: B,
}

impl<A, B> Add<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Add<Output = A::Output>,
{
    pub fn new(a: A, b: B) -> Self {
        Add { a, b }
    }
}

impl<A, B> Node for Add<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Add<Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        self.a.proc(ctx) + self.b.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.a.lock(ctx);
        self.b.lock(ctx);
    }

    fn unlock(&mut self) {
        self.a.unlock();
        self.b.unlock();
    }
}
