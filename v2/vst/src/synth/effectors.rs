use corus_v2::{
    nodes::{
        biquad_filter::BiquadFilter,
        effects::{chorus::Chorus, phaser::Phaser, DelayFx, EarlyReflections, SchroederReverb},
        mix::mix,
    },
    signal::StereoF64,
    ProcessContext,
};

pub enum Effector {
    Filter { frequency: f64, q: f64 },
    Phaser,
    Chorus,
    Delay,
    Reverb,
    Gain { gain: f64 },
    Tanh,
}

pub enum State {
    Filter {
        filter: BiquadFilter<2, StereoF64>,
    },
    Phaser {
        phaser: Phaser<StereoF64>,
    },
    Chorus {
        chorus: Chorus<StereoF64>,
    },
    Delay {
        delay: DelayFx<StereoF64>,
    },
    Reverb {
        reverb: SchroederReverb<StereoF64>,
        er: EarlyReflections<StereoF64>,
    },
    Gain,
    Tanh,
    None,
}

impl Effector {
    pub fn name(&self) -> &'static str {
        match self {
            Effector::Filter { .. } => "Filter",
            Effector::Phaser { .. } => "Phaser",
            Effector::Chorus { .. } => "Chorus",
            Effector::Delay { .. } => "Delay",
            Effector::Reverb { .. } => "Reverb",
            Effector::Gain { .. } => "Gain",
            Effector::Tanh => "Tanh",
        }
    }

    pub fn process(&self, state: &mut State, ctx: &ProcessContext, x: StereoF64) -> StereoF64 {
        match (self, state) {
            (Effector::Filter { frequency, q }, State::Filter { filter }) => {
                filter.process(ctx, *frequency, *q, x)
            }
            (Effector::Phaser, State::Phaser { phaser }) => phaser.process(ctx, x),
            (Effector::Chorus, State::Chorus { chorus }) => chorus.process(ctx, 0.001, 0.001, x),
            (Effector::Delay, State::Delay { delay }) => delay.process(ctx, x, 0.5, 0.3, 0.2),
            (Effector::Reverb, State::Reverb { reverb, er }) => mix(&[
                (0.8, x),
                (0.3, er.process(ctx, x)),
                (0.2, reverb.process(ctx, x)),
            ]),
            (Effector::Gain { gain }, State::Gain) => x * *gain,
            (Effector::Tanh, State::Tanh) => x.map(|x| x.tanh()),
            _ => unreachable!("invalid state"),
        }
    }

    pub fn ensure_state(&self, state: &mut State) {
        match (self, state) {
            (Effector::Filter { .. }, State::Filter { .. }) => {}
            (Effector::Phaser, State::Phaser { .. }) => {}
            (Effector::Chorus, State::Chorus { .. }) => {}
            (Effector::Delay, State::Delay { .. }) => {}
            (Effector::Reverb, State::Reverb { .. }) => {}
            (Effector::Gain { .. }, State::Gain) => {}
            (Effector::Tanh, State::Tanh) => {}
            (Effector::Filter { .. }, state) => {
                *state = State::Filter {
                    filter: BiquadFilter::new(),
                }
            }
            (Effector::Phaser, state) => {
                *state = State::Phaser {
                    phaser: Phaser::new(),
                }
            }
            (Effector::Chorus, state) => {
                *state = State::Chorus {
                    chorus: Chorus::new(),
                }
            }
            (Effector::Delay, state) => {
                *state = State::Delay {
                    delay: DelayFx::new(48000),
                }
            }
            (Effector::Reverb, state) => {
                *state = State::Reverb {
                    reverb: SchroederReverb::new(48000),
                    er: EarlyReflections::new(),
                }
            }
            (Effector::Gain { .. }, state) => {
                *state = State::Gain;
            }
            (Effector::Tanh, state) => {
                *state = State::Tanh;
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::None
    }
}
