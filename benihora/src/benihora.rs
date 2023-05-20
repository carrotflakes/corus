use super::glottis::Glottis;
use super::interval_timer::IntervalTimer;
use super::noise::Noise;
use super::tract::Tract;
use super::F;

pub struct Benihora {
    pub(crate) aspiration_noise: Noise,
    pub(crate) fricative_noise: Noise,
    pub(crate) sample_rate: F,
    pub glottis: Glottis,
    pub tract: Tract,
    pub(crate) update_timer: IntervalTimer,
    pub(crate) sound_speed: usize,
}

impl Benihora {
    pub fn new(sound_speed: usize, sample_rate: F) -> Self {
        Self {
            aspiration_noise: Noise::new(1, sample_rate, 500.0),
            fricative_noise: Noise::new(2, sample_rate, 1000.0),
            sample_rate,
            glottis: Glottis::new(),
            tract: Tract::new(),
            update_timer: IntervalTimer::new_overflowed(0.04),
            sound_speed,
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

        if self.update_timer.overflowed() {
            self.tract.update_block(self.update_timer.interval);
        }
        let lambda = self.update_timer.progress();
        self.update_timer.update(1.0 / self.sample_rate as f64);

        let glottal_output = self.glottis.process(
            current_time,
            1.0 / self.sample_rate as f64,
            aspiration_noise,
            frequency,
            tenseness,
            intensity,
            loudness,
        );
        let glottal_output = glottal_output + aspiration_noise * 1.0e-16; // avoid subnormal
        let mut vocal_out = 0.0;
        for i in 0..self.sound_speed {
            let time = current_time + i as f64 / self.sound_speed as f64 / self.sample_rate as f64;
            let (mouth, nose) = self.tract.run_step(
                time,
                glottal_output,
                fricative_noise,
                lambda,
                1.0 / (self.sample_rate as usize * self.sound_speed) as f64,
            );
            vocal_out += mouth + nose;
        }
        (vocal_out / self.sound_speed as f64).into()
    }
}
