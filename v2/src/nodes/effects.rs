use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

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
