use num_traits::{Float, FromPrimitive};

use crate::ProccessContext;

pub struct Sine<F: Float + FromPrimitive> {
    phase: F,
}

impl<F: Float + FromPrimitive> Sine<F> {
    pub fn new() -> Self {
        Self { phase: F::zero() }
    }

    pub fn process(&mut self, ctx: &ProccessContext, frequency: F) -> F {
        let dphase = frequency * F::from_f64(ctx.dtime() * std::f64::consts::TAU).unwrap();
        let x = self.phase.sin();
        self.phase = self.phase + dphase;
        x
    }
}
