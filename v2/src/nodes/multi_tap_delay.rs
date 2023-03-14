use crate::{ring_buffer::RingBuffer, signal::SignalExt, ProccessContext};

use num_traits::*;

pub struct MultiTapDelay<S: SignalExt>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    buffer: RingBuffer<S>,
}

impl<S: SignalExt> MultiTapDelay<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, taps: &[(S::Float, S)], x: S) -> S {
        let mut y = S::default();
        for (delay, gain) in taps {
            let i = (*delay * S::Float::from_f64(ctx.sample_rate()).unwrap())
                .to_usize()
                .unwrap();
            y = y.add(self.buffer.get(i).mul(*gain));
        }
        self.buffer.push(x);
        y
    }
}
