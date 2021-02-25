use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::signal::{C1f64, IntoStereo, Signal};

use super::{Node, ProcContext};

pub struct Pan<F, FP, T, O, A, B, DA, DB>
where
    FP: Clone + 'static,
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<FP> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    a: DA,
    b: DB,
    _t: (
        PhantomData<FP>,
        PhantomData<T>,
        PhantomData<O>,
        PhantomData<A>,
        PhantomData<B>,
    ),
}

impl<F, FP, T, O, A, B, DA, DB> Pan<F, FP, T, O, A, B, DA, DB>
where
    FP: Clone + 'static,
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<FP> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new(a: DA, b: DB) -> Self {
        Pan {
            a,
            b,
            _t: Default::default(),
        }
    }
}

impl<T, O, A, B, DA, DB> Node<O> for Pan<f64, C1f64, T, O, A, B, DA, DB>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<f64, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = f64>,
    A: Node<T> + ?Sized,
    B: Node<C1f64> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> O {
        let v: T = self.a.as_mut().proc(ctx);
        let pan = self.b.as_mut().proc(ctx);
        v.into_stereo_with_pan(pan.get(0))
    }

    fn lock(&mut self) {
        self.a.as_mut().lock();
        self.b.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.a.as_mut().unlock();
        self.b.as_mut().unlock();
    }
}

impl<F, FP, T, O, A, B, DA, DB> AsMut<Self> for Pan<F, FP, T, O, A, B, DA, DB>
where
    FP: Clone + 'static,
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<FP> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn as_mut(&mut self) -> &mut Pan<F, FP, T, O, A, B, DA, DB> {
        self
    }
}
