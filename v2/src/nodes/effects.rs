use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

use super::{all_pass_filter::AllPassFilter, comb_filter::CombFilter};

pub struct DelayFx<S: SignalExt> {
    buffer: RingBuffer<S>,
}

impl<S: SignalExt> DelayFx<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProccessContext,
        x: S,
        delay: S::Float,
        feedback: S::Float,
    ) -> S {
        let i = (delay * S::Float::from_f64(ctx.sample_rate()).unwrap())
            .to_usize()
            .unwrap();
        let y = x.add(self.buffer.get(i).mul(S::from_float(feedback)));
        self.buffer.push(y);
        y
    }
}

pub struct SchroederReverb<S: SignalExt> {
    combs: [(S::Float, CombFilter<S>); 4],
    all_passes: [(S::Float, S::Float, AllPassFilter<S>); 2],
}

impl<S: SignalExt> SchroederReverb<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            combs: [
                (S::Float::from_f64(0.03).unwrap(), CombFilter::new(len)),
                (S::Float::from_f64(0.031).unwrap(), CombFilter::new(len)),
                (S::Float::from_f64(0.034).unwrap(), CombFilter::new(len)),
                (S::Float::from_f64(0.036).unwrap(), CombFilter::new(len)),
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
        for (delay, comb) in self.combs.iter_mut() {
            y = y.add(comb.process(ctx, x, *delay, S::Float::from_f64(0.8).unwrap()));
        }

        for (delay, feedback, all_pass) in self.all_passes.iter_mut() {
            y = all_pass.process(ctx, y, *delay, *feedback);
        }

        x.add(y.mul(S::from_float(S::Float::from_f64(0.3).unwrap())))
    }
}
