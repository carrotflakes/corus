use crate::{ring_buffer::RingBuffer, signal::Signal};

use super::{Node, ProcContext};

pub struct CombFilter<A>
where
    A: Node,
    A::Output: Signal,
{
    node: A,
    pub delay: f64,
    pub gain: A::Output,
    buffer: RingBuffer<A::Output>,
}

impl<A> CombFilter<A>
where
    A: Node,
    A::Output: Signal,
{
    pub fn new(node: A, delay: f64, gain: A::Output) -> Self {
        CombFilter {
            node,
            delay,
            gain,
            buffer: RingBuffer::new(1),
        }
    }
}

impl<A> Node for CombFilter<A>
where
    A: Node,
    A::Output: Signal,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<A> std::borrow::Borrow<RingBuffer<A::Output>> for CombFilter<A>
where
    A: Node,
    A::Output: Signal,
{
    fn borrow(&self) -> &RingBuffer<A::Output> {
        &self.buffer
    }
}
