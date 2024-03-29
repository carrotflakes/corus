use crate::{signal::Signal, ProcessContext};

pub struct Impulse<S: Signal> {
    value: S,
}

impl<S: Signal> Impulse<S> {
    pub fn new() -> Self {
        Self {
            value: S::default(),
        }
    }

    pub fn set(&mut self, value: S) {
        self.value = value;
    }

    pub fn process(&mut self, _ctx: &ProcessContext) -> S {
        let x = self.value;
        self.value = S::default();
        x
    }
}
