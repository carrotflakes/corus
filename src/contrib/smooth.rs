use std::ops::{Add, Mul, Neg};

use crate::{proc_context::ProcContext, signal::{C1f64, Signal}};

use super::Node;

pub struct Smooth<T, A, DA>
where
    T: Signal + Mul<Output = T> + Add<Output = T> + Neg + Default + Clone,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    level: T,
    value: T,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA> Smooth<T, A, DA>
where
    T: Signal + Mul<Output = T> + Add<Output = T> + Neg + Default + Clone,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA, level: T) -> Self {
        Smooth {
            node,
            level,
            value: Default::default(),
            _t: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<A, DA> Node<C1f64> for Smooth<C1f64, A, DA>
where
    A: Node<C1f64> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let v = self.node.as_mut().proc(ctx);
        self.value = self.value * self.level + v * (1.0 - self.level);
        self.value
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, A, DA> AsMut<Self> for Smooth<T, A, DA>
where
    T: Signal + Mul<Output = T> + Add<Output = T> + Neg + Default + Clone,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
