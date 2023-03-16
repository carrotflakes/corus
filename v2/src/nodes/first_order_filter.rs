use crate::{signal::Signal, ProcessContext};

pub struct FirstOrderLowPassFilter<S: Signal> {
    prev: S,
}

impl<S: Signal> FirstOrderLowPassFilter<S> {
    pub fn new() -> Self {
        Self { prev: S::default() }
    }

    pub fn process(&mut self, _ctx: &ProcessContext, k: S::Float, x: S) -> S {
        self.prev = self.prev.add(S::from(k).mul(x.sub(self.prev)));
        self.prev
    }
}
