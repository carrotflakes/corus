use crate::{proc_context::ProcContext};

use super::Node;

pub struct Crossfader<A, B, C>
where
    A: Node,
    B: Node<Output = A::Output>,
    C: Node,
    A::Output: CrossfaderLevel<C::Output>,
{
    a: A,
    b: B,
    level: C,
}

impl<A, B, C> Crossfader<A, B, C>
where
    A: Node,
    B: Node<Output = A::Output>,
    C: Node,
    A::Output: CrossfaderLevel<C::Output>,
{
    pub fn new(a: A, b: B, level: C) -> Self {
        Crossfader { a, b, level }
    }
}

impl<A, B, C> Node for Crossfader<A, B, C>
where
    A: Node,
    B: Node<Output = A::Output>,
    C: Node,
    A::Output: CrossfaderLevel<C::Output>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let a = self.a.proc(ctx);
        let b = self.b.proc(ctx);
        let level = self.level.proc(ctx);
        a.lerp(b, level)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.a.lock(ctx);
        self.b.lock(ctx);
        self.level.lock(ctx);
    }

    fn unlock(&mut self) {
        self.a.unlock();
        self.b.unlock();
        self.level.unlock();
    }
}

pub trait CrossfaderLevel<F>: 'static + Clone + Default + From<f64> + Into<f64> {
    fn lerp(&self, other: Self, r: F) -> Self;
}

impl CrossfaderLevel<f64> for f64 {
    #[inline]
    fn lerp(&self, other: Self, r: f64) -> Self {
        self * (1.0 - r) + other * r
    }
}
