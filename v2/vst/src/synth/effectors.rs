use corus_v2::{
    nodes::{
        biquad_filter::BiquadFilter,
        effects::{
            chorus::Chorus,
            compressor::{Compressor, Param as CompressorParam},
            phaser::Phaser,
            DelayFx, EarlyReflections, SchroederReverb,
        },
        mix::mix,
    },
    signal::StereoF64,
    ProcessContext,
};
use serde::{Deserialize, Serialize};

use super::param_f64::{self, EnvelopeState};

// pub enum MonoEffector {
//     Filter { frequency: f64, q: f64 },
//     Delay,
//     Gain { gain: f64 },
//     Tanh,
// }

// pub enum MonoState {
//     Filter {
//         filter: BiquadFilter<2, StereoF64>,
//     },
//     Delay {
//         delay: DelayFx<StereoF64>,
//     },
//     Gain,
//     Tanh,
//     None,
// }

#[derive(Serialize, Deserialize)]
pub enum Effector {
    Filter {
        frequency: param_f64::ParamF64,
        q: param_f64::ParamF64,
    },
    Phaser,
    Chorus,
    Delay,
    Reverb,
    Gain {
        gain: param_f64::ParamF64,
    },
    Compressor {
        threshold: param_f64::ParamF64,
        ratio: param_f64::ParamF64,
        attack: param_f64::ParamF64,
        release: param_f64::ParamF64,
        gain: param_f64::ParamF64,
    },
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
    Compressor {
        compressor: Compressor<f64>,
    },
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
            Effector::Compressor { .. } => "Compressor",
            Effector::Tanh => "Tanh",
        }
    }

    pub fn param_names(&self) -> &[&'static str] {
        match self {
            Effector::Filter { .. } => &["frequency", "q"],
            Effector::Phaser { .. } => &[],
            Effector::Chorus { .. } => &[],
            Effector::Delay { .. } => &[],
            Effector::Reverb { .. } => &[],
            Effector::Gain { .. } => &["gain"],
            Effector::Compressor { .. } => &["threshold", "ratio", "attack", "release", "gain"],
            Effector::Tanh => &[],
        }
    }

    pub fn param_muts<'a>(&'a mut self) -> Vec<&'a mut param_f64::ParamF64> {
        match self {
            Effector::Filter { frequency, q } => vec![frequency, q],
            Effector::Phaser { .. } => vec![],
            Effector::Chorus { .. } => vec![],
            Effector::Delay { .. } => vec![],
            Effector::Reverb { .. } => vec![],
            Effector::Gain { gain } => vec![gain],
            Effector::Compressor {
                threshold,
                ratio,
                attack,
                release,
                gain,
            } => vec![threshold, ratio, attack, release, gain],
            Effector::Tanh => vec![],
        }
    }

    pub fn process(
        &self,
        state: &mut State,
        ctx: &ProcessContext,
        env_state: &EnvelopeState,
        x: StereoF64,
    ) -> StereoF64 {
        match (self, state) {
            (Effector::Filter { frequency, q }, State::Filter { filter }) => filter.process(
                ctx,
                frequency.compute(env_state).max(20.0),
                q.compute(env_state),
                x,
            ),
            (Effector::Phaser, State::Phaser { phaser }) => phaser.process(ctx, x),
            (Effector::Chorus, State::Chorus { chorus }) => chorus.process(ctx, 0.001, 0.001, x),
            (Effector::Delay, State::Delay { delay }) => delay.process(ctx, x, 0.5, 0.3, 0.2),
            (Effector::Reverb, State::Reverb { reverb, er }) => mix(&[
                (0.8, x),
                (0.3, er.process(ctx, x)),
                (0.2, reverb.process(ctx, x)),
            ]),
            (Effector::Gain { gain }, State::Gain) => x * gain.compute(env_state),
            (
                Effector::Compressor {
                    threshold,
                    ratio,
                    attack,
                    release,
                    gain,
                },
                State::Compressor { compressor },
            ) => {
                let threshold = threshold.compute(env_state);
                let ratio = ratio.compute(env_state);
                let attack = attack.compute(env_state);
                let release = release.compute(env_state);
                let gain = gain.compute(env_state);
                compressor.process(
                    &CompressorParam {
                        threshold,
                        ratio,
                        attack,
                        release,
                        gain,
                    },
                    ctx,
                    x,
                )
            }
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
            (Effector::Compressor { .. }, State::Compressor { .. }) => {}
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
            (Effector::Compressor { .. }, state) => {
                *state = State::Compressor {
                    compressor: Compressor::new(),
                }
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
