use crate::{
    signal::{IntoStereo, StereoF64},
    ProcessContext,
};

use super::phase::Phase;

pub struct Unison {
    pub phases: Vec<Phase<f64>>,
}

impl Unison {
    pub fn new(n: usize) -> Self {
        Self {
            phases: (0..n).map(|_| Phase::new()).collect(),
        }
    }

    pub fn set_voice_num(&mut self, n: usize) {
        self.phases.resize_with(n, Phase::new);
    }

    pub fn reset(&mut self) {
        for phase in self.phases.iter_mut() {
            phase.set(0.0);
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProcessContext,
        frequency: f64,
        detune: f64,
        stereo_width: f64,
        f: impl Fn(f64) -> f64,
    ) -> StereoF64 {
        let n = self.phases.len();
        let scale = 1.0 / (n as f64).sqrt();
        let mut x = StereoF64::default();
        for (i, phase) in self.phases.iter_mut().enumerate() {
            let detune_amount = if n == 1 {
                0.0
            } else {
                detune * (i as f64 / n as f64 - 0.5)
            };
            let frequency = frequency * (1.0 + detune_amount);
            let phase = phase.process(ctx, frequency);
            let y = f(phase) * scale;
            let pan = if n == 1 {
                0.0
            } else {
                (i as f64 / (n - 1) as f64 - 0.5) * 2.0 * stereo_width
            };
            // TODO: dry/wet
            x = x + y.into_stereo_with_pan(pan);
        }
        x
    }
}
