use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::signal::{IntoStereo, Signal};

use super::{Node, ProcContext};

pub struct Pan<O, A, B>
where
    O: Clone + 'static + Signal<Float = f64>,
    A: Node,
    A::Output: Clone
        + 'static
        + Add<Output = A::Output>
        + Mul<Output = A::Output>
        + IntoStereo<Output = O>,
    B: Node<Output = f64>,
{
    a: A,
    b: B,
    _t: PhantomData<O>,
}

impl<O, A, B> Pan<O, A, B>
where
    O: Clone + 'static + Signal<Float = f64>,
    A: Node,
    A::Output: Clone
        + 'static
        + Add<Output = A::Output>
        + Mul<Output = A::Output>
        + IntoStereo<Output = O>,
    B: Node<Output = f64>,
{
    pub fn new(a: A, b: B) -> Self {
        Pan {
            a,
            b,
            _t: Default::default(),
        }
    }
}

impl<O, A, B> Node for Pan<O, A, B>
where
    O: Clone + 'static + Signal<Float = f64>,
    A: Node,
    A::Output: Signal<Float = f64> + IntoStereo<Output = O>,
    B: Node<Output = f64>,
{
    type Output = O;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> O {
        let v: A::Output = self.a.proc(ctx);
        let pan = self.b.proc(ctx);
        v.into_stereo_with_pan(pan)
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
