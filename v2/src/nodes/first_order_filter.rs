use crate::{signal::Signal, ProcessContext};

pub struct LowPassFilter<S: Signal> {
    prev: S,
}

impl<S: Signal> LowPassFilter<S> {
    pub fn new() -> Self {
        Self { prev: S::default() }
    }

    pub fn process(&mut self, _ctx: &ProcessContext, k: S::Float, x: S) -> S {
        // TODO: k???
        self.prev = self.prev + (x - self.prev) * k;
        self.prev
    }
}

pub struct HighPassFilter<S: Signal> {
    prev_x: S,
    prev: S,
}

impl<S: Signal> HighPassFilter<S> {
    pub fn new() -> Self {
        Self {
            prev_x: S::default(),
            prev: S::default(),
        }
    }

    pub fn process(&mut self, _ctx: &ProcessContext, k: S::Float, x: S) -> S {
        self.prev = (self.prev + x - self.prev_x) * k;
        self.prev_x = x;
        self.prev
    }
}

pub struct AllPassFilter<S: Signal> {
    prev: S,
}

impl<S: Signal> AllPassFilter<S> {
    pub fn new() -> Self {
        Self { prev: S::default() }
    }

    pub fn process(&mut self, _ctx: &ProcessContext, k: S::Float, x: S) -> S {
        let x = x + self.prev * k;
        let y = self.prev - x * k;
        self.prev = x;
        y
    }
}
