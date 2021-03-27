use crate::{ring_buffer::RingBuffer, signal::Signal, time::AsSample};

use super::{Node, ProcContext};

pub struct Rms<A, B>
where
    A: Node,
    A::Output: Signal<Float = f64> + Default,
    B: AsSample,
{
    node: A,
    buffer: RingBuffer<A::Output>,
    size: B,
    acc: A::Output,
}

impl<A, B> Rms<A, B>
where
    A: Node,
    A::Output: Signal<Float = f64> + Default,
    B: AsSample,
{
    pub fn new(node: A, size: B) -> Self {
        Rms {
            node,
            buffer: RingBuffer::new(1),
            size,
            acc: A::Output::default(),
        }
    }
}

impl<A, B> Node for Rms<A, B>
where
    A: Node,
    A::Output: Signal<Float = f64> + Default,
    B: AsSample,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let window_size = self.size.as_sample(ctx.sample_rate) as usize;
        let squared_v = self.node.proc(ctx).map(|x| x.powi(2));
        self.acc = self.acc.clone() + squared_v.clone() + -self.buffer.get(window_size);
        self.buffer.push(squared_v);
        (self.acc.clone() / window_size as f64).map(|x| x.sqrt())
    }

    fn lock(&mut self, ctx: &ProcContext) {
        let window_size = self.size.as_sample(ctx.sample_rate) as usize;
        self.buffer.fast_resize(window_size + 1);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
