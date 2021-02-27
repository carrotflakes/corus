use crate::{signal::Signal, Node, ProcContext};

pub struct DownSample<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    node: A,
    value: T,
    sample_rate: u64,
    next_update: f64,
}

impl<T, A> DownSample<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    pub fn new(node: A, value: T, sample_rate: u64) -> Self {
        DownSample {
            node,
            value,
            sample_rate,
            next_update: 0.0,
        }
    }
}

impl<T, A> Node<T> for DownSample<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.next_update <= ctx.time {
            self.value = self.node.proc(&ProcContext {
                sample_rate: self.sample_rate,
                time: ctx.time,
            });
            self.next_update += 1.0 / self.sample_rate as f64;
        }
        self.value.clone()
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
