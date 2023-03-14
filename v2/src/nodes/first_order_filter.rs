use crate::{signal::SignalExt, ProccessContext};

pub struct FirstOrderLowPassFilter<S: SignalExt> {
    prev: S,
}

impl<S: SignalExt> FirstOrderLowPassFilter<S> {
    pub fn new() -> Self {
        Self { prev: S::default() }
    }

    pub fn process(&mut self, _ctx: &ProccessContext, k: S::Float, x: S) -> S {
        self.prev = self.prev.add(S::from_float(k).mul(x.sub(self.prev)));
        self.prev
    }
}
