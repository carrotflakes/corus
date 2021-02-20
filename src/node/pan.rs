use std::ops::{Add, Mul};

use crate::signal::{C1f32, IntoStereo, Signal};

use super::{Node, ProcContext};

pub struct Pan<F, T, O, A, B, DA, DB>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    a: DA,
    b: DB,
    _t: std::marker::PhantomData<T>,
    _o: std::marker::PhantomData<O>,
    _a: std::marker::PhantomData<A>,
    _b: std::marker::PhantomData<B>,
}

impl<F, T, O, A, B, DA, DB> Pan<F, T, O, A, B, DA, DB>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new(a: DA, b: DB) -> Self {
        Pan {
            a,
            b,
            _t: Default::default(),
            _o: Default::default(),
            _a: Default::default(),
            _b: Default::default(),
        }
    }
}

impl<F, T, O, A, B, DA, DB> Node<O> for Pan<F, T, O, A, B, DA, DB>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> O {
        let v: T = self.a.as_mut().proc(ctx);
        let pan = self.b.as_mut().proc(ctx);
        v.into_stereo_with_pan(pan.0[0])
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

impl<F, T, O, A, B, DA, DB> AsMut<Self> for Pan<F, T, O, A, B, DA, DB>
where
    T: Clone + 'static + Add<Output = T> + Mul<Output = T> + IntoStereo<F, Output = O>,
    O: Clone + 'static + Add<Output = O> + Mul<Output = O> + Signal<Float = F>,
    A: Node<T> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn as_mut(&mut self) -> &mut Pan<F, T, O, A, B, DA, DB> {
        self
    }
}
