use super::{Node, ProcContext};

pub struct Sine {
    frequency: Box<dyn Node<f32>>,
}

impl Sine {
    pub fn new(frequency: Box<dyn Node<f32>>) -> Self {
        Sine {
            frequency,
        }
    }
}

impl Node<f32> for Sine {
    fn procedure(&self) -> Box<dyn FnMut(&ProcContext) -> f32> {
        let mut frequency = self.frequency.procedure();
        let mut phase = 0.0;
        Box::new(move |ctx| {
            let f = frequency(ctx);
            let p = phase;
            phase += f / ctx.sample_rate as f32;
            phase = phase.fract();
            (std::f32::consts::PI * 2.0 * p).sin()
        })
    }
}
