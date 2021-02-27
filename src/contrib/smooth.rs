use std::ops::{Add, Mul, Neg};

use crate::{proc_context::ProcContext, signal::{C1f64, Signal}};

use super::Node;

pub struct Smooth<T, A>
where
    T: Signal + Mul<Output = T> + Add<Output = T> + Neg + Default + Clone,
    A: Node<T>,
{
    node: A,
    level: T,
    value: T,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> Smooth<T, A>
where
    T: Signal + Mul<Output = T> + Add<Output = T> + Neg + Default + Clone,
    A: Node<T>,
{
    pub fn new(node: A, level: T) -> Self {
        Smooth {
            node,
            level,
            value: Default::default(),
            _t: Default::default(),
        }
    }
}

impl<A> Node<C1f64> for Smooth<C1f64, A>
where
    A: Node<C1f64>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let v = self.node.proc(ctx);
        self.value = self.value * self.level + v * (1.0 - self.level);
        self.value
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
