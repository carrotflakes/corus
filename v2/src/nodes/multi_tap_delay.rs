use crate::{interpolate_get, ring_buffer::RingBuffer, signal::Signal, ProcessContext};

use num_traits::*;

pub struct MultiTapDelay<S: Signal>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    buffer: RingBuffer<S>,
}

impl<S: Signal> MultiTapDelay<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
        }
    }

    pub fn process(&mut self, ctx: &ProcessContext, taps: &[(S::Float, S)], x: S) -> S {
        let mut y = S::default();
        for (delay, gain) in taps {
            let i = *delay * S::Float::from_f64(ctx.sample_rate()).unwrap();
            let v = interpolate_get(i, |i| self.buffer.get(i));
            y = y + v * *gain;
        }
        self.buffer.push(x);
        y
    }
}
