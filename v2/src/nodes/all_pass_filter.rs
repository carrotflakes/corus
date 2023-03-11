use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

pub struct AllPassFilter<S: SignalExt> {
    buffer: RingBuffer<S>,
}

impl<S: SignalExt> AllPassFilter<S>
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
        let feedback = S::from_float(feedback);
        let i = (delay * S::Float::from_f64(ctx.sample_rate()).unwrap())
            .to_usize()
            .unwrap();
        let d = self.buffer.get(i);
        let a = x.add(d.mul(feedback));
        self.buffer.push(a);
        d.sub(a.mul(feedback))
    }
}
