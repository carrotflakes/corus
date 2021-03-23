use crate::{ring_buffer::RingBuffer, signal::Signal};

use super::{Node, ProcContext};

pub struct Sma<A>
where
    A: Node,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    node: A,
    duration: f64,
    buffer: RingBuffer<A::Output>,
    acc: A::Output,
    size: usize,
}

impl<A> Sma<A>
where
    A: Node,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    pub fn new(node: A, duration: f64) -> Self {
        Sma {
            node,
            duration,
            buffer: RingBuffer::new(0),
            acc: A::Output::default(),
            size: 0,
        }
    }
}

impl<A> Node for Sma<A>
where
    A: Node,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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
        self.buffer
            .fast_resize((self.duration * ctx.sample_rate as f64) as usize + 1);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
