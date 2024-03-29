use crate::{interpolate_get, ring_buffer::RingBuffer, signal::Signal, ProcessContext};

use num_traits::*;

/// FIR all-pass filter.
pub struct AllPassFilter<S: Signal> {
    buffer: RingBuffer<S>,
}

impl<S: Signal> AllPassFilter<S>
where
    S::Float: FromPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProcessContext,
        x: S,
        delay: S::Float,
        feedback: S::Float,
    ) -> S {
        let feedback = S::from(feedback);
        let i = delay * S::Float::from_f64(ctx.sample_rate()).unwrap();
        let d = interpolate_get(i, |i| self.buffer.get(i));
        let a = x + d * feedback;
        self.buffer.push(a);
        d - a * feedback
    }
}
