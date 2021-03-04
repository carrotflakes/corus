use std::marker::PhantomData;

use crate::proc_context::ProcContext;

use super::Node;

pub struct Crossfader<F, T, A, B, C>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T>,
    B: Node<T>,
    C: Node<F>,
{
    a: A,
    b: B,
    level: C,
    _t: (
        PhantomData<F>,
        PhantomData<T>,
    ),
}

impl<F, T, A, B, C> Crossfader<F, T, A, B, C>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T>,
    B: Node<T>,
    C: Node<F>,
{
    pub fn new(a: A, b: B, level: C) -> Self {
        Crossfader {
            a,
            b,
            level,
            _t: Default::default(),
        }
    }
}

impl<F, T, A, B, C> Node<T> for Crossfader<F, T, A, B, C>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T>,
    B: Node<T>,
    C: Node<F>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let a = self.a.proc(ctx);
        let b = self.b.proc(ctx);
        let level = self.level.proc(ctx);
        a.lerp(b, level)
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

pub trait CrossfaderLevel<F>: 'static + Clone + Default + From<f64> + Into<f64> {
    fn lerp(&self, other: Self, r: F) -> Self;
}

impl CrossfaderLevel<f64> for f64 {
    #[inline]
    fn lerp(&self, other: Self, r: f64) -> Self {
        self * (1.0 - r) + other * r
    }
}
