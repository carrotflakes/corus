use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct RingBufferRecord<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    node: A,
    buffer: RingBuffer<A::Output>,
}

impl<A> RingBufferRecord<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    pub fn new(node: A, size: usize) -> Self {
        RingBufferRecord {
            node,
            buffer: RingBuffer::new(size),
        }
    }

    pub fn get_buffer(&self) -> &RingBuffer<A::Output> {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut RingBuffer<A::Output> {
        &mut self.buffer
    }
}

impl<A> Node for RingBufferRecord<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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

impl<A> std::borrow::Borrow<RingBuffer<A::Output>> for RingBufferRecord<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    fn borrow(&self) -> &RingBuffer<A::Output> {
        &self.buffer
    }
}
