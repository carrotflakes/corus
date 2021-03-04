use std::borrow::Borrow;

use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct RingBufferPlayback<T, A, B>
where
    T: 'static + Clone + Default,
    A: Node<f64>,
    B: Borrow<RingBuffer<T>>,
{
    node: A,
    buffer: B,
    _t: std::marker::PhantomData<T>,
}

impl<T, A, B> RingBufferPlayback<T, A, B>
where
    T: 'static + Clone + Default,
    A: Node<f64>,
    B: Borrow<RingBuffer<T>>,
{
    pub fn new(node: A, buffer: B) -> Self {
        RingBufferPlayback {
            node,
            buffer,
            _t: Default::default(),
        }
    }
}

impl<T, A, B> Node<T> for RingBufferPlayback<T, A, B>
where
    T: 'static + Clone + Default,
    A: Node<f64>,
    B: Borrow<RingBuffer<T>>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let t = self.node.proc(ctx);
        let i = (t * ctx.sample_rate as f64).round() as usize;
        self.buffer.borrow().get(i)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
