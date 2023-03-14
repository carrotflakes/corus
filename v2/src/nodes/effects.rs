use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

use super::{
    all_pass_filter::AllPassFilter, comb_filter::CombFilter,
    first_order_filter::FirstOrderLowPassFilter,
};

pub struct DelayFx<S: SignalExt> {
    buffer: RingBuffer<S>,
    filter: FirstOrderLowPassFilter<S>,
}

impl<S: SignalExt> DelayFx<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
            filter: FirstOrderLowPassFilter::new(),
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProccessContext,
        x: S,
        delay: S::Float,
        feedback: S::Float,
        low_pass: S::Float,
    ) -> S {
        let i = (delay * S::Float::from_f64(ctx.sample_rate()).unwrap())
            .to_usize()
            .unwrap();
        let d = self.buffer.get(i).mul(S::from_float(feedback));
        let y = x.add(self.filter.process(ctx, low_pass, d));
        self.buffer.push(y);
        y
    }
}

pub struct SchroederReverb<S: SignalExt> {
    combs: [(S::Float, S::Float, CombFilter<S>); 4],
    all_passes: [(S::Float, S::Float, AllPassFilter<S>); 2],
}

impl<S: SignalExt> SchroederReverb<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            combs: [
                (
                    S::Float::from_f64(0.03).unwrap(),
                    S::Float::from_f64(0.83).unwrap(),
                    CombFilter::new(len),
                ),
                (
                    S::Float::from_f64(0.031).unwrap(),
                    S::Float::from_f64(0.8).unwrap(),
                    CombFilter::new(len),
                ),
                (
                    S::Float::from_f64(0.034).unwrap(),
                    S::Float::from_f64(0.76).unwrap(),
                    CombFilter::new(len),
                ),
                (
                    S::Float::from_f64(0.039).unwrap(),
                    S::Float::from_f64(0.7).unwrap(),
                    CombFilter::new(len),
                ),
            ],
            all_passes: [
                (
                    S::Float::from_f64(0.0011).unwrap(),
                    S::Float::from_f64(0.7).unwrap(),
                    AllPassFilter::new(len),
                ),
                (
                    S::Float::from_f64(0.0043).unwrap(),
                    S::Float::from_f64(0.6).unwrap(),
                    AllPassFilter::new(len),
                ),
            ],
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, x: S) -> S {
        let mut y = S::default();
        for (delay, feedback, comb) in self.combs.iter_mut() {
            y = y.add(comb.process(ctx, x, *delay, *feedback));
        }

        for (delay, feedback, all_pass) in self.all_passes.iter_mut() {
            y = all_pass.process(ctx, y, *delay, *feedback);
        }

        y
    }
}
