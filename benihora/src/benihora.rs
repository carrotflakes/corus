use crate::resample::Resample;

use super::glottis::Glottis;
use super::noise::Noise;
use super::tract::Tract;
use super::F;

pub struct Benihora {
    inner_sample_rate: F,
    pub sample_rate: F,
    pub(crate) aspiration_noise: Noise,
    pub(crate) fricative_noise: Noise,
    pub glottis: Glottis,
    pub tract: Tract,
    resample: Resample,
}

impl Benihora {
    pub fn new(inner_sample_rate: F, sample_rate: F, seed: u32) -> Self {
        assert!(seed < u32::MAX - 2);
        Self {
            inner_sample_rate,
            sample_rate,
            aspiration_noise: Noise::new(seed + 1, sample_rate, 500.0),
            fricative_noise: Noise::new(seed + 2, sample_rate, 1000.0),
            glottis: Glottis::new(),
            tract: Tract::new(1, inner_sample_rate),
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

        let aspiration_noise = self.aspiration_noise.process();
        let fricative_noise = self.fricative_noise.process();

        self.resample.process(|| {
            let glottal_output = self.glottis.process(
                current_time,
                1.0 / self.inner_sample_rate as f64,
                aspiration_noise,
                frequency,
                tenseness,
                intensity,
                loudness,
            );

            // Add a bit of noise to avoid subnormal
            let glottal_output = glottal_output + aspiration_noise * 1.0e-16;

            self.tract
                .process(current_time, fricative_noise, glottal_output)
        })
    }
}
