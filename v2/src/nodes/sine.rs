use crate::ProccessContext;

pub struct Sine {
    phase: f64,
}

impl Sine {
    pub fn new() -> Self {
        Self { phase: 0.0 }
    }

    pub fn process(&mut self, ctx: &ProccessContext, frequency: f64) -> f64 {
        let dphase = frequency * ctx.dtime() * std::f64::consts::TAU;
        let x = self.phase.sin();
        self.phase += dphase;
        x
    }
}
