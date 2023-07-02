use crate::resample::Resample;

use super::glottis::Glottis;
use super::tract::Tract;
use super::F;

pub struct Benihora {
    pub sample_rate: F,
    pub glottis: Glottis,
    pub tract: Tract,
    resample: Resample,
}

impl Benihora {
    pub fn new(sound_speed: F, sample_rate: F, over_sample: F, seed: u32) -> Self {
        assert!(seed < u32::MAX - 2);

        let tract_steps = 48000.0 * sound_speed;
        let tract_steps_per_process = ((tract_steps / sample_rate) as usize).max(1);
        let inner_sample_rate = tract_steps / tract_steps_per_process as F * over_sample;

        Self {
            sample_rate,
            glottis: Glottis::new(inner_sample_rate, seed),
            tract: Tract::new(tract_steps_per_process, inner_sample_rate, seed + 1),
            resample: Resample::new(inner_sample_rate, sample_rate),
        }
    }

    pub fn process(
        &mut self,
        current_time: F,
        frequency: F,
        tenseness: F,
        intensity: F,
        loudness: F,
    ) -> F {
        debug_assert!((1.0..=10000.0).contains(&frequency));
        debug_assert!((0.0..=1.0).contains(&tenseness));
        debug_assert!((0.0..=1.0).contains(&intensity));
        debug_assert!((0.0..=1.0).contains(&loudness));

        self.resample.process(|| {
            let glottal_output = self
                .glottis
                .process(frequency, tenseness, intensity, loudness);

            self.tract.process(current_time, glottal_output)
        })
    }
}
