use std::marker::PhantomData;

use crate::proc_context::ProcContext;

use super::Node;

pub struct Crossfader<F, T, A, B, C, DA, DB, DC>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    C: Node<F> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    a: DA,
    b: DB,
    level: DC,
    _t: (
        PhantomData<F>,
        PhantomData<T>,
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
    ),
}

impl<F, T, A, B, C, DA, DB, DC> Crossfader<F, T, A, B, C, DA, DB, DC>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    C: Node<F> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    pub fn new(a: DA, b: DB, level: DC) -> Self {
        Crossfader {
            a,
            b,
            level,
            _t: Default::default(),
        }
    }
}

impl<F, T, A, B, C, DA, DB, DC> Node<T> for Crossfader<F, T, A, B, C, DA, DB, DC>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    C: Node<F> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let a = self.a.as_mut().proc(ctx);
        let b = self.b.as_mut().proc(ctx);
        let level = self.level.as_mut().proc(ctx);
        a.lerp(b, level)
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

impl<F, T, A, B, C, DA, DB, DC> AsMut<Self> for Crossfader<F, T, A, B, C, DA, DB, DC>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    C: Node<F> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Crossfader<F, T, A, B, C, DA, DB, DC> {
        self
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
