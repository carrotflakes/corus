use crate::{ring_buffer::RingBuffer, signal::Signal};

use super::{Node, ProcContext};

pub enum Interpolation {
    NearestNeighbor,
    Bilinear,
}

pub struct Delay<A, B>
where
    A: Node,
    B: Node<Output = f64>,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    node: A,
    delay: B,
    buffer: RingBuffer<A::Output>,
    interpolation: Interpolation,
}

impl<A, B> Delay<A, B>
where
    A: Node,
    B: Node<Output = f64>,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    pub fn new(node: A, delay: B, size: usize, interpolation: Interpolation) -> Self {
        Delay {
            node,
            delay,
            buffer: RingBuffer::new(size),
            interpolation,
        }
    }
}

impl<A, B> Node for Delay<A, B>
where
    A: Node,
    B: Node<Output = f64>,
    A::Output: Signal + Default,
    <A::Output as Signal>::Float: From<f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let v = self.node.proc(ctx);
        let delay = self.delay.proc(ctx);
        self.buffer.push(v.clone());
        match self.interpolation {
            Interpolation::NearestNeighbor => self.buffer.get(delay.round() as usize),
            Interpolation::Bilinear => {
                let delay_i = delay.floor() as usize;
                self.buffer.get(delay_i).lerp(
                    &self.buffer.get(delay_i + 1),
                    <A::Output as Signal>::Float::from(delay.fract()),
                )
            }
        }
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
        self.delay.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
        self.delay.unlock();
    }
}
