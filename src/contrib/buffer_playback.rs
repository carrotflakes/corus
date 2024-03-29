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

impl<T, B> Node for BufferPlayback<T, B>
where
    T: 'static + Clone + Default,
    B: Borrow<Vec<T>>,
{
    type Output = T;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let s = (ctx.current_time * ctx.sample_rate as f64) as usize;
        let buf = self.buffer.borrow();
        buf[s % buf.len()].clone()
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}
