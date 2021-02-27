use std::ops::{Add, Mul};

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct CombFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T>,
{
    node: A,
    pub delay: f64,
    pub gain: T,
    buffer: RingBuffer<T>,
}

impl<T, A> CombFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T>,
{
    pub fn new(node: A, delay: f64, gain: T) -> Self {
        CombFilter {
            node,
            delay,
            gain,
            buffer: RingBuffer::new(0),
        }
    }
}

impl<T, A> Node<T> for CombFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let delay_len = (self.delay * ctx.sample_rate as f64) as usize;
        let desire_buffer_len = delay_len + 1;
        if self.buffer.size() != desire_buffer_len {
            self.buffer.fast_resize(desire_buffer_len);
        }

        let delay_value = self.buffer.get(delay_len) * self.gain.clone();

        let v = self.node.proc(ctx) + delay_value;
        self.buffer.push(v.clone());
        v
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<T, A> std::borrow::Borrow<RingBuffer<T>> for CombFilter<T, A>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.buffer
    }
}
