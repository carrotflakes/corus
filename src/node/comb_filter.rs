use std::ops::{Add, Mul};

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct CombFilter<T, A, DA>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    pub delay: f32,
    pub gain: T,
    buffer: RingBuffer<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA> CombFilter<T, A, DA>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA, delay: f32, gain: T) -> Self {
        CombFilter {
            node,
            delay,
            gain,
            buffer: RingBuffer::new(0),
            _a: Default::default(),
        }
    }
}

impl<T, A, DA> Node<T> for CombFilter<T, A, DA>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let delay_len = (self.delay * ctx.sample_rate as f32) as usize;
        let desire_buffer_len = delay_len + 1;
        if self.buffer.size() != desire_buffer_len {
            self.buffer.fast_resize(desire_buffer_len);
        }

        let delay_value = self.buffer.get(delay_len) * self.gain.clone();

        let v = self.node.as_mut().proc(ctx) + delay_value;
        self.buffer.push(v.clone());
        v
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, A, DA> AsMut<Self> for CombFilter<T, A, DA>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, A, DA> std::borrow::Borrow<RingBuffer<T>> for CombFilter<T, A, DA>
where
    T: 'static + Clone + Default + Mul<Output = T> + Add<Output = T>,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.buffer
    }
}
