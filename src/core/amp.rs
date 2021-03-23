use std::ops::Mul;

use crate::signal::Signal;

use super::{Node, ProcContext};

pub struct Amp<A, B>
where
    A: Node,
    B: Node<Output = <<A as Node>::Output as Signal>::Float>,
    A::Output: Signal + Mul<B::Output, Output = A::Output>,
{
    input: A,
    gain: B,
}

impl<A, B> Amp<A, B>
where
    A: Node,
    B: Node<Output = <<A as Node>::Output as Signal>::Float>,
    A::Output: Signal + Mul<B::Output, Output = A::Output>,
{
    pub fn new(input: A, gain: B) -> Self {
        Amp { input, gain }
    }
}

impl<A, B> Node for Amp<A, B>
where
    A: Node,
    B: Node<Output = <<A as Node>::Output as Signal>::Float>,
    A::Output: Signal + Mul<B::Output, Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> A::Output {
        self.input.proc(ctx) * self.gain.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.input.lock(ctx);
        self.gain.lock(ctx);
    }

    fn unlock(&mut self) {
        self.input.unlock();
        self.gain.unlock();
    }
}
