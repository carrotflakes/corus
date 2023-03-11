use crate::{
    signal::{IntoStereo, SignalExt, StereoF64},
    ProccessContext,
};

use super::phase::Phase;

pub struct Unison {
    pub phases: Vec<Phase>,
}

impl Unison {
    pub fn new(n: usize) -> Self {
        Self {
            phases: (0..n).map(|_| Phase::new()).collect(),
        }
    }

    pub fn reset(&mut self) {
        for phase in self.phases.iter_mut() {
            phase.set(0.0);
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProccessContext,
        frequency: f64,
        detune: f64,
        stereo_width: f64,
        f: impl Fn(f64) -> f64,
    ) -> StereoF64 {
        let n = self.phases.len();
        let scale = 1.0 / n as f64;
        let mut x = StereoF64::default();
        for (i, phase) in self.phases.iter_mut().enumerate() {
            let frequency = frequency * (1.0 + detune * (i as f64 / n as f64 - 0.5));
            let phase = phase.process(ctx, frequency);
            let y = f(phase) * scale;
            let pan = (i as f64 / n as f64 - 0.5) * 2.0 * stereo_width;
            x = x.add(y.into_stereo_with_pan(pan));
        }
        x
    }
}
