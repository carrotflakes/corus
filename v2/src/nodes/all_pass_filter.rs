use crate::{ring_buffer::RingBuffer, signal::Signal, ProccessContext};

use num_traits::*;

pub struct AllPassFilter<S: Signal> {
    buffer: RingBuffer<S>,
}

impl<S: Signal> AllPassFilter<S>
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
        let feedback = S::from(feedback);
        let i = (delay * S::Float::from_f64(ctx.sample_rate()).unwrap())
            .to_usize()
            .unwrap();
        let d = self.buffer.get(i);
        let a = x + d * feedback;
        self.buffer.push(a);
        d - a * feedback
    }
}
