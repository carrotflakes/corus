use crate::{
    proc_context::ProcContext,
    signal::{C1f64, C2f64, IntoMono, Mono, Signal},
};

use super::Node;

pub struct SimpleComp<T, A, B, C, D>
where
    T: Signal,
    A: Node<Output = T>,
    B: Node<Output = <A::Output as Signal>::Float>,
    C: Node<Output = <A::Output as Signal>::Float>,
    D: Node<Output = <A::Output as Signal>::Float>,
{
    node: A,
    threshold: B,
    ratio: C,
    out_gain: D,
}

impl<T, A, B, C, D> SimpleComp<T, A, B, C, D>
where
    T: Signal,
    A: Node<Output = T>,
    B: Node<Output = <A::Output as Signal>::Float>,
    C: Node<Output = <A::Output as Signal>::Float>,
    D: Node<Output = <A::Output as Signal>::Float>,
{
    pub fn new(node: A, threshold: B, ratio: C, out_gain: D) -> Self {
        Self {
            node,
            threshold,
            ratio,
            out_gain,
        }
    }
}

impl<A, B, C, D> Node for SimpleComp<C1f64, A, B, C, D>
where
    A: Node<Output = C1f64>,
    B: Node<Output = f64>,
    C: Node<Output = f64>,
    D: Node<Output = f64>,
{
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<A, B, C, D> Node for SimpleComp<C2f64, A, B, C, D>
where
    A: Node<Output = C2f64>,
    B: Node<Output = f64>,
    C: Node<Output = f64>,
    D: Node<Output = f64>,
{
    type Output = C2f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
