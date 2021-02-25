use std::borrow::Borrow;

use crate::{core::Node, proc_context::ProcContext};

pub struct BufferPlayback<T, B>
where
    T: 'static + Clone + Default,
    B: Borrow<Vec<T>>,
{
    buffer: B,
    _t: std::marker::PhantomData<T>,
}

impl<T, B> BufferPlayback<T, B>
where
    T: 'static + Clone + Default,
    B: Borrow<Vec<T>>,
{
    pub fn new(buffer: B) -> Self {
        BufferPlayback {
            buffer,
            _t: Default::default(),
        }
    }
}

impl<T, B> Node<T> for BufferPlayback<T, B>
where
    T: 'static + Clone + Default,
    B: Borrow<Vec<T>>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let s = (ctx.time * ctx.sample_rate as f64) as usize;
        let buf = self.buffer.borrow();
        buf[s % buf.len()].clone()
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl<T, B> AsMut<Self> for BufferPlayback<T, B>
where
    T: 'static + Clone + Default,
    B: Borrow<Vec<T>>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
