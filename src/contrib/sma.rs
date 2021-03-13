use crate::{ring_buffer::RingBuffer, signal::Signal};

use super::{Node, ProcContext};

pub struct Sma<T, A>
where
    T: Signal + Default,
    A: Node<T>,
    <T as Signal>::Float: From<f64>,
{
    node: A,
    duration: f64,
    buffer: RingBuffer<T>,
    acc: T,
    size: usize,
}

impl<T, A> Sma<T, A>
where
    T: Signal + Default,
    A: Node<T>,
    <T as Signal>::Float: From<f64>,
{
    pub fn new(node: A, duration: f64) -> Self {
        Sma {
            node,
            duration,
            buffer: RingBuffer::new(0),
            acc: T::default(),
            size: 0,
        }
    }
}

impl<T, A> Node<T> for Sma<T, A>
where
    T: Signal + Default,
    A: Node<T>,
    <T as Signal>::Float: From<f64>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let samples = (self.duration * ctx.sample_rate as f64) as usize;
        let v = self.node.proc(ctx);
        if self.size >= samples {
            self.acc = self.acc.clone() + v.clone() + -self.buffer.get(samples);
        } else {
            self.acc = self.acc.clone() + v.clone();
            self.size = (self.size + 1).min(samples);
        }
        self.buffer.push(v);
        self.acc.clone() / self.size as f64
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.buffer.fast_resize((self.duration * ctx.sample_rate as f64) as usize + 1);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
