use std::ops::{Add, Mul, Neg};

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct AllPassFilter<A>
where
    A: Node,
    A::Output: 'static + Clone + Default + Mul<Output = A::Output> + Add<Output = A::Output> + Neg<Output = A::Output>,
{
    node: A,
    pub delay: f32,
    pub gain: A::Output,
    buffer: RingBuffer<A::Output>,
}

impl<A> AllPassFilter<A>
where
    A: Node,
    A::Output: 'static + Clone + Default + Mul<Output = A::Output> + Add<Output = A::Output> + Neg<Output = A::Output>,
{
    pub fn new(node: A, delay: f32, gain: A::Output) -> Self {
        AllPassFilter {
            node,
            delay,
            gain,
            buffer: RingBuffer::new(1),
        }
    }
}

impl<A> Node for AllPassFilter<A>
where
    A: Node,
    A::Output: 'static + Clone + Default + Mul<Output = A::Output> + Add<Output = A::Output> + Neg<Output = A::Output>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> A::Output {
        let delay_len = (self.delay * ctx.sample_rate as f32) as usize;
        let desire_buffer_len = delay_len + 1;
        if self.buffer.size() != desire_buffer_len {
            self.buffer.fast_resize(desire_buffer_len);
        }

        let delay_value = self.buffer.get(delay_len);

        let v = self.node.proc(ctx) + delay_value.clone() * self.gain.clone();
        self.buffer.push(v.clone());
        delay_value + v * -self.gain.clone()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<A> std::borrow::Borrow<RingBuffer<A::Output>> for AllPassFilter<A>
where
    A: Node,
    A::Output: 'static + Clone + Default + Mul<Output = A::Output> + Add<Output = A::Output> + Neg<Output = A::Output>,
{
    fn borrow(&self) -> &RingBuffer<A::Output> {
        &self.buffer
    }
}
