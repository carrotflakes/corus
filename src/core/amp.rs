use std::ops::Mul;

use crate::signal::Signal;

use super::{Node, ProcContext};

pub struct Amp<F, T, A, B>
where
    F: 'static,
    T: Signal<Float = F> + Mul<F, Output = T>,
    A: Node<T>,
    B: Node<F>,
{
    input: A,
    gain: B,
    _t: std::marker::PhantomData<T>,
}

impl<F, T, A, B> Amp<F, T, A, B>
where
    F: 'static,
    T: Signal<Float = F> + Mul<F, Output = T>,
    A: Node<T>,
    B: Node<F>,
{
    pub fn new(input: A, gain: B) -> Self {
        Amp {
            input,
            gain,
            _t: Default::default(),
        }
    }
}

impl<F, T, A, B> Node<T> for Amp<F, T, A, B>
where
    F: 'static,
    T: Signal<Float = F> + Mul<F, Output = T>,
    A: Node<T>,
    B: Node<F>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
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
