use crate::{iterator::SampleIterator, node::Node};

pub struct ProcContext {
    pub sample_rate: u64,
    pub time: f64,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            time: 0.0,
        }
    }

    #[inline]
    pub fn sample<T: 'static, N: Node<T> + ?Sized>(&mut self, mut node: impl AsMut<N>) -> T {
        let r = node.as_mut().proc(self);
        self.time += 1.0 / self.sample_rate as f64;
        r
    }

    pub fn into_iter<T: 'static, A: Node<T>, DA: AsMut<A>>(self, node: DA) -> SampleIterator<T, A, DA> {
        SampleIterator::new(self, node)
    }
}
