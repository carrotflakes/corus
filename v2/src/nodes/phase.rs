use crate::ProccessContext;

pub struct Phase {
    phase: f64,
}

impl Phase {
    pub fn new() -> Self {
        Self { phase: 0.0 }
    }

    pub fn process(&mut self, ctx: &ProccessContext, frequency: f64) -> f64 {
        let dphase = frequency * ctx.dtime();
        let phase = self.phase;
        self.phase = (self.phase + dphase).fract();
        phase
    }
}
