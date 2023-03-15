use crate::{interpolate_get, ring_buffer::RingBuffer, signal::Signal, ProccessContext};

use num_traits::*;

pub struct CombFilter<S: Signal> {
    buffer: RingBuffer<S>,
}

impl<S: Signal> CombFilter<S>
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
        let i = delay * S::Float::from_f64(ctx.sample_rate()).unwrap();
        let y = interpolate_get(i, |i| self.buffer.get(i));
        self.buffer.push(x + y * S::from(feedback));
        y
    }
}
