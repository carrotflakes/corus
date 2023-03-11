use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

pub struct CombFilter<S: SignalExt> {
    buffer: RingBuffer<S>,
}

impl<S: SignalExt> CombFilter<S>
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
        let y = self.buffer.get(i);
        self.buffer.push(x.add(y.mul(S::from_float(feedback))));
        y
    }
}
