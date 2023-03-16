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
    Filter {
        frequency: f64,
        q: f64,
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
    Gain {
        gain: f64,
    },
    Tanh,
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

    pub fn process(&mut self, ctx: &ProcessContext, x: StereoF64) -> StereoF64 {
        match self {
            Effector::Filter {
                filter,
                frequency,
                q,
            } => filter.process(ctx, *frequency, *q, x),
            Effector::Phaser { phaser } => phaser.process(ctx, x),
            Effector::Chorus { chorus } => chorus.process(ctx, 0.001, 0.001, x),
            Effector::Delay { delay } => delay.process(ctx, x, 0.5, 0.3, 0.2),
            Effector::Reverb { reverb, er } => mix(&[
                (0.8, x),
                (0.3, er.process(ctx, x)),
                (0.2, reverb.process(ctx, x)),
            ]),
            Effector::Gain { gain } => x * *gain,
            Effector::Tanh => x.map(|x| x.tanh()),
        }
    }
}
