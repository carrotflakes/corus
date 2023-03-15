use crate::{nodes::multi_tap_delay::MultiTapDelay, signal::Signal, ProccessContext};
use num_traits::{FromPrimitive, ToPrimitive};

use super::super::sine::Sine;

pub struct Chorus<S: Signal>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    multi_tap_delay: MultiTapDelay<S>,
    lfos: [Sine<S::Float>; 3],
}

impl<S: Signal> Chorus<S>
where
    S::Float: FromPrimitive + ToPrimitive,
{
    pub fn new() -> Self {
        Self {
            multi_tap_delay: MultiTapDelay::new(44100),
            lfos: [Sine::new(), Sine::new(), Sine::new()],
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, delay: S::Float, depth: S::Float, x: S) -> S {
        self.multi_tap_delay.process(
            ctx,
            &[
                (
                    delay
                        + depth
                            * (S::float_from_f64(0.5)
                                + self.lfos[0].process(ctx, S::Float::from_f64(0.5).unwrap())
                                    * S::float_from_f64(0.5)),
                    S::from(S::float_from_f64(0.3)),
                ),
                (
                    delay
                        + depth
                            * (S::float_from_f64(0.5)
                                + self.lfos[0].process(ctx, S::Float::from_f64(0.56).unwrap())
                                    * S::float_from_f64(0.5)),
                    S::from(S::float_from_f64(0.3)),
                ),
                (
                    delay
                        + depth
                            * (S::float_from_f64(0.5)
                                + self.lfos[0].process(ctx, S::Float::from_f64(0.81).unwrap())
                                    * S::float_from_f64(0.5)),
                    S::from(S::float_from_f64(0.3)),
                ),
            ],
            x,
        ) + x
    }
}
