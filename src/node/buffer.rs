use crate::ring_buffer::RingBuffer;

use super::{Node, ProcContext};

pub struct Buffer<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    buffer: RingBuffer<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA> Buffer<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA, size: usize) -> Self {
        Buffer {
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

impl<T, A, DA> Node<T> for Buffer<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let v = self.node.as_mut().proc(ctx);
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

impl<T, A, DA> AsMut<Self> for Buffer<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, A, DA> std::borrow::Borrow<RingBuffer<T>> for Buffer<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.buffer
    }
}
