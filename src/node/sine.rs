use super::{Node, ProcContext};

pub struct Sine {
    frequency: Box<dyn Node<f32>>,
    phase: f32,
}

impl Sine {
    pub fn new(frequency: Box<dyn Node<f32>>) -> Self {
        Sine {
            frequency,
            phase: 0.0,
        }
    }
}

impl Node<f32> for Sine {
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        let f = self.frequency.proc(ctx);
        let p = self.phase;
        self.phase = (self.phase + f / ctx.sample_rate as f32).fract();
        (std::f32::consts::PI * 2.0 * p).sin()
    }
}
