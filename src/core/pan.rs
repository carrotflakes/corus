use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::signal::{C1f64, IntoStereo, Signal};

use super::{Node, ProcContext};

pub struct Pan<F, FP, T, O, A, B>
where
    FP: Clone + 'static,
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T>,
    B: Node<FP>,
{
    a: A,
    b: B,
    _t: (
        PhantomData<FP>,
        PhantomData<T>,
        PhantomData<O>,
    ),
}

impl<F, FP, T, O, A, B> Pan<F, FP, T, O, A, B>
where
    FP: Clone + 'static,
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T>,
    B: Node<FP>,
{
    pub fn new(a: A, b: B) -> Self {
        Pan {
            a,
            b,
            _t: Default::default(),
        }
    }
}

impl<T, O, A, B> Node<O> for Pan<f64, C1f64, T, O, A, B>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<f64, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = f64>,
    A: Node<T>,
    B: Node<C1f64>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> O {
        let v: T = self.a.proc(ctx);
        let pan = self.b.proc(ctx);
        v.into_stereo_with_pan(pan.get(0))
    }

    fn lock(&mut self) {
        self.a.lock();
        self.b.lock();
    }

    fn unlock(&mut self) {
        self.a.unlock();
        self.b.unlock();
    }
}

