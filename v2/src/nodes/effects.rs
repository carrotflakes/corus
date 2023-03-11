use crate::{ring_buffer::RingBuffer, ProccessContext};

pub struct DelayFx {
    buffer: RingBuffer<f64>,
}

impl DelayFx {
    pub fn new(len: usize) -> Self {
        Self {
            buffer: RingBuffer::new(len),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, x: f64, delay: f64, feedback: f64) -> f64 {
        let i = (delay * ctx.sample_rate()) as usize;
        let y = x + self.buffer.get(i) * feedback;
        self.buffer.push(y);
        y
    }
}
