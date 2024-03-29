use num_traits::{Float, FromPrimitive};

use crate::ProcessContext;

pub struct Phase<F: Float + FromPrimitive> {
    phase: F,
}

impl<F: Float + FromPrimitive> Phase<F> {
    pub fn new() -> Self {
        Self { phase: F::zero() }
    }

    pub fn set(&mut self, phase: F) {
        self.phase = phase.fract();
    }

    pub fn process(&mut self, ctx: &ProcessContext, frequency: F) -> F {
        let dphase = frequency * F::from_f64(ctx.dtime()).unwrap();
        let phase = self.phase;
        self.phase = (self.phase + dphase).fract();
        phase
    }

    /// Returns the current phase and next phase.
    /// The next phase can overflow 1.0.
    pub fn process_range(&mut self, ctx: &ProcessContext, frequency: F) -> (F, F) {
        let dphase = frequency * F::from_f64(ctx.dtime()).unwrap();
        let phase = self.phase;
        self.phase = (self.phase + dphase).fract();
        (phase, phase + dphase)
    }
}
