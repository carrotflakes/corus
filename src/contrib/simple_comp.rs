use std::marker::PhantomData;

use crate::{
    proc_context::ProcContext,
    signal::{C1f64, C2f64, IntoMono, Mono, Signal},
};

use super::Node;

pub struct SimpleComp<F, T, A, B, C, D>
where
    F: 'static,
    T: Signal<Float = F>,
    A: Node<T>,
    B: Node<F>,
    C: Node<F>,
    D: Node<F>,
{
    node: A,
    threshold: B,
    ratio: C,
    out_gain: D,
    _t: (PhantomData<F>, PhantomData<T>),
}

impl<F, T, A, B, C, D> SimpleComp<F, T, A, B, C, D>
where
    F: 'static,
    T: Signal<Float = F>,
    A: Node<T>,
    B: Node<F>,
    C: Node<F>,
    D: Node<F>,
{
    pub fn new(node: A, threshold: B, ratio: C, out_gain: D) -> Self {
        Self {
            node,
            threshold,
            ratio,
            out_gain,
            _t: Default::default(),
        }
    }
}

impl<A, B, C, D> Node<C1f64> for SimpleComp<f64, C1f64, A, B, C, D>
where
    A: Node<C1f64>,
    B: Node<f64>,
    C: Node<f64>,
    D: Node<f64>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let v = self.node.proc(ctx).get_m();
        let threshold = self.threshold.proc(ctx);
        let ratio = self.ratio.proc(ctx);
        let out_gain = self.out_gain.proc(ctx);
        out_gain
            * if threshold < v {
                threshold + (v - threshold) * ratio
            } else if v < -threshold {
                -threshold + (v - -threshold) * ratio
            } else {
                v
            }
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<A, B, C, D> Node<C2f64> for SimpleComp<f64, C2f64, A, B, C, D>
where
    A: Node<C2f64>,
    B: Node<f64>,
    C: Node<f64>,
    D: Node<f64>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C2f64 {
        let v = self.node.proc(ctx);
        let threshold = self.threshold.proc(ctx);
        let ratio = self.ratio.proc(ctx);
        let out_gain = self.out_gain.proc(ctx);
        let abs = v.into_mono().abs();
        let gain = if threshold < abs {
            threshold / (threshold + (abs - threshold) * ratio)
        } else {
            1.0
        };
        v * (gain * out_gain)
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
