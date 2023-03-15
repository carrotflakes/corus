use crate::{signal::Signal, ProccessContext};
use num_traits::FromPrimitive;

use super::super::{all_pass_filter::AllPassFilter, sine::Sine};

pub struct Phaser<S: Signal>
where
    S::Float: FromPrimitive,
{
    all_pass_filters: Vec<AllPassFilter<S>>,
    lfo: Sine<S::Float>,
    prev: S,
}

impl<S: Signal> Phaser<S>
where
    S::Float: FromPrimitive,
{
    pub fn new() -> Self {
        Self {
            all_pass_filters: (0..4).map(|_| AllPassFilter::new(44100)).collect(),
            lfo: Sine::new(),
            prev: S::zero(),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, x: S) -> S {
        let feedback = S::from(S::Float::from_f64(0.5).unwrap());
        let depth = S::Float::from_f64(2.0).unwrap();
        let mut y = x + self.prev * feedback;
        let sin = self.lfo.process(ctx, S::Float::from_f64(1.0).unwrap());
        // Is this correct?
        let f = S::Float::from_f64(1000.0).unwrap()
            * (S::Float::from_f64(1.0).unwrap()
                + (sin * S::Float::from_f64(0.5).unwrap() + S::Float::from_f64(0.5).unwrap())
                    * depth);
        let t = S::Float::from_f64(1.0).unwrap() / f;
        for all_pass_filter in &mut self.all_pass_filters {
            y = all_pass_filter.process(ctx, y, t, S::Float::from_f64(0.1).unwrap());
        }
        self.prev = y;
        // TODO: dry/wet
        x + y
    }
}
