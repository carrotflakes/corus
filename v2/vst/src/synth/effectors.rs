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
use wavetables::shapers;

use super::{param_f64, param_pool::ParamPool};

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

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub enum ShaperType {
    HardClip,
    Tanh,
    Sin,
    Wrap,
    Triangle,
}

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
    Shaper {
        pre_gain: param_f64::ParamF64,
        r#type: ShaperType,
    },
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
    Shaper,
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
            Effector::Shaper { .. } => "Shaper",
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
            Effector::Shaper { .. } => &["pre_gain"],
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
            Effector::Shaper { pre_gain, .. } => vec![pre_gain],
        }
    }

    pub fn process(
        &self,
        state: &mut State,
        ctx: &ProcessContext,
        param_pools: &[&ParamPool],
        x: StereoF64,
    ) -> StereoF64 {
        match (self, state) {
            (Effector::Filter { frequency, q }, State::Filter { filter }) => filter.process(
                ctx,
                frequency.compute(param_pools).clamp(20.0, 20000.0),
                q.compute(param_pools),
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
            (Effector::Gain { gain }, State::Gain) => x * gain.compute(param_pools),
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
                let threshold = threshold.compute(param_pools);
                let ratio = ratio.compute(param_pools);
                let attack = attack.compute(param_pools);
                let release = release.compute(param_pools);
                let gain = gain.compute(param_pools);
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
            (Effector::Shaper { pre_gain, r#type }, State::Shaper) => {
                let pre_gain = pre_gain.compute(param_pools);
                let x = x * pre_gain;
                match r#type {
                    ShaperType::HardClip => x.map(|x| shapers::hard_clip(x)),
                    ShaperType::Tanh => x.map(|x| shapers::tanh(x)),
                    ShaperType::Sin => x.map(|x| shapers::sin(x)),
                    ShaperType::Wrap => x.map(|x| shapers::wrap(x)),
                    ShaperType::Triangle => x.map(|x| shapers::triangle(x)),
                }
            }
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
            (Effector::Shaper { .. }, state) => {
                *state = State::Shaper;
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::None
    }
}

impl ShaperType {
    pub const ALL: [ShaperType; 5] = [
        ShaperType::HardClip,
        ShaperType::Tanh,
        ShaperType::Sin,
        ShaperType::Wrap,
        ShaperType::Triangle,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ShaperType::HardClip => "HardClip",
            ShaperType::Tanh => "Tanh",
            ShaperType::Sin => "Sin",
            ShaperType::Wrap => "Wrap",
            ShaperType::Triangle => "Triangle",
        }
    }
}
