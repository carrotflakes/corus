use std::borrow::Borrow;

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct RingBufferPlayback<T, A, DA, B>
where
    T: 'static + Clone + Default,
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
    B: Borrow<RingBuffer<T>>,
{
    node: DA,
    buffer: B,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA, B> RingBufferPlayback<T, A, DA, B>
where
    T: 'static + Clone + Default,
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
    B: Borrow<RingBuffer<T>>,
{
    pub fn new(node: DA, buffer: B) -> Self {
        RingBufferPlayback {
            node,
            buffer,
            _t: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<T, A, DA, B> Node<T> for RingBufferPlayback<T, A, DA, B>
where
    T: 'static + Clone + Default,
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
    B: Borrow<RingBuffer<T>>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let t = self.node.as_mut().proc(ctx);
        let i = (t * ctx.sample_rate as f32).round() as usize;
        self.buffer.borrow().get(i)
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, A, DA, B> AsMut<Self> for RingBufferPlayback<T, A, DA, B>
where
    T: 'static + Clone + Default,
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
    B: Borrow<RingBuffer<T>>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
