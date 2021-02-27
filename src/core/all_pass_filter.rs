use std::ops::{Add, Mul, Neg};

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct AllPassFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T> + Neg,
    A: Node<T>,
{
    node: A,
    pub delay: f32,
    pub gain: T,
    buffer: RingBuffer<T>,
}

impl<T, A> AllPassFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T> + Neg<Output = T>,
    A: Node<T>,
{
    pub fn new(node: A, delay: f32, gain: T) -> Self {
        AllPassFilter {
            node,
            delay,
            gain,
            buffer: RingBuffer::new(0),
        }
    }
}

impl<T, A> Node<T> for AllPassFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T> + Neg<Output = T>,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
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

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<T, A> std::borrow::Borrow<RingBuffer<T>> for AllPassFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T> + Neg<Output = T>,
    A: Node<T>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.buffer
    }
}
