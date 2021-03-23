use super::{Node, ProcContext};

pub struct Mul<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Mul<Output = A::Output>,
{
    input1: A,
    input2: B,
}

impl<A, B> Mul<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Mul<Output = A::Output>,
{
    pub fn new(input1: A, input2: B) -> Self {
        Mul { input1, input2 }
    }
}

impl<A, B> Node for Mul<A, B>
where
    A: Node,
    B: Node<Output = A::Output>,
    A::Output: Clone + 'static + std::ops::Mul<Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        self.input1.proc(ctx) * self.input2.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.input1.lock(ctx);
        self.input2.lock(ctx);
    }

    fn unlock(&mut self) {
        self.input1.unlock();
        self.input2.unlock();
    }
}
