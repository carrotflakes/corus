use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct RingBufferRecord<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    node: A,
    buffer: RingBuffer<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A> RingBufferRecord<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    pub fn new(node: A, size: usize) -> Self {
        RingBufferRecord {
            node,
            buffer: RingBuffer::new(size),
            _a: Default::default(),
        }
    }

    pub fn get_buffer(&self) -> &RingBuffer<T> {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut RingBuffer<T> {
        &mut self.buffer
    }
}

impl<T, A> Node<T> for RingBufferRecord<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let v = self.node.proc(ctx);
        self.buffer.push(v.clone());
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<T, A> std::borrow::Borrow<RingBuffer<T>> for RingBufferRecord<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.buffer
    }
}
