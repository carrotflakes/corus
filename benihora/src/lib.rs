mod glottis;
mod interval_timer;
mod noise;
mod tract;

use std::f64::consts::TAU;

pub use glottis::Glottis;
use interval_timer::IntervalTimer;
use noise::Noise;
pub use tract::{Constriction, Mouth, Nose, Tract};

type F = f64;

pub struct Benihora {
    pub sound: bool,
    pub frequency: Frequency,
    tenseness: Tenseness,
    intensity: F,
    loudness: F,

    aspiration_noise: Noise,
    fricative_noise: Noise,
    sample_rate: F,
    pub glottis: Glottis,
    pub tract: Tract,
    update_timer: IntervalTimer,
    sound_speed: usize,
}

impl Benihora {
    pub fn new(sound_speed: usize) -> Self {
        let sample_rate = 48000.0;
        Self {
            sound: true,
            frequency: Frequency::new(140.0, 0.005, 6.0),
            tenseness: Tenseness::new(0.6),
            intensity: 0.0,
            loudness: 1.0,
            aspiration_noise: Noise::new(1, sample_rate, 500.0),
            fricative_noise: Noise::new(2, sample_rate, 1000.0),
            sample_rate,
            glottis: Glottis::new(),
            tract: Tract::new(),
            update_timer: IntervalTimer::new_overflowed(0.04),
            sound_speed,
        }
    }

    /// let v = v.clamp(0.0, 1.0);
    /// set_tenseness(1.0 - (v * std::f64::consts::PI * 0.5).cos());
    pub fn set_tenseness(&mut self, tenseness: F) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness = tenseness.powf(0.25);
    }

    fn set_sample_rate(&mut self, sample_rate: F) {
        if sample_rate != self.sample_rate {
            self.sample_rate = sample_rate;
            self.aspiration_noise = Noise::new(1, sample_rate, 500.0);
            self.fricative_noise = Noise::new(2, sample_rate, 1000.0);
        }
    }

    pub fn process(&mut self, current_time: F, sample_rate: F) -> F {
        self.set_sample_rate(sample_rate);
        let aspiration_noise = self.aspiration_noise.process();
        let fricative_noise = self.fricative_noise.process();

        if self.update_timer.overflowed() {
            if self.sound {
                self.intensity += self.update_timer.interval * 3.25;
            } else {
                self.intensity -= self.update_timer.interval * 5.0;
            }
            self.intensity = self.intensity.clamp(0.0, 1.0);

            self.frequency.update(current_time);
            self.tenseness.update(current_time);
            self.tract.update_block(self.update_timer.interval);
        }
        self.update_timer.update(1.0 / sample_rate as f64);

        let lambda = self.update_timer.progress();
        let frequency = self.frequency.get(lambda);
        let tenseness = self.tenseness.get(lambda);
        let glottal_output = self.glottis.compute(
            current_time,
            1.0 / sample_rate as f64,
            aspiration_noise,
            frequency,
            tenseness,
            self.intensity,
            self.loudness,
        );
        let glottal_output = glottal_output + aspiration_noise * 1.0e-16; // avoid subnormal
        let mut vocal_out = 0.0;
        for i in 0..self.sound_speed {
            let time = current_time + i as f64 / self.sound_speed as f64 / sample_rate as f64;
            let (mouth, nose) = self.tract.run_step(
                time,
                glottal_output,
                fricative_noise,
                lambda,
                1.0 / (sample_rate as usize * self.sound_speed) as f64,
            );
            vocal_out += mouth + nose;
        }
        (vocal_out / self.sound_speed as f64).into()
    }
}

fn simplex1(x: F) -> F {
    perlin_noise::perlin_noise([x * 1.2, -x * 0.7, 0.0])
}

#[inline]
fn lerp(a: F, b: F, t: F) -> F {
    a + (b - a) * t
}

pub struct Frequency {
    old_frequency: F,
    new_frequency: F,
    target_frequency: F,
    smooth_frequency: F,

    pub vibrato_amount: F,
    pub vibrato_frequency: F,
}

impl Frequency {
    fn new(frequency: F, vibrato_amount: F, vibrato_frequency: F) -> Self {
        Self {
            old_frequency: frequency,
            new_frequency: frequency,
            target_frequency: frequency,
            smooth_frequency: frequency,
            vibrato_amount,
            vibrato_frequency,
        }
    }

    pub fn set(&mut self, frequency: F) {
        self.target_frequency = frequency;
    }

    fn update(&mut self, time: F) {
        let mut vibrato = self.vibrato_amount * (TAU * time * self.vibrato_frequency).sin();
        vibrato += 0.02 * simplex1(time * 4.07);
        vibrato += 0.04 * simplex1(time * 2.15);

        self.smooth_frequency = (self.smooth_frequency + self.target_frequency) * 0.5;

        self.old_frequency = self.new_frequency;
        self.new_frequency = self.smooth_frequency * (1.0 + vibrato);
    }

    fn get(&self, lambda: F) -> F {
        lerp(self.old_frequency, self.new_frequency, lambda)
    }
}

pub struct Tenseness {
    old_tenseness: F,
    new_tenseness: F,
    target_tenseness: F,
}

impl Tenseness {
    fn new(tenseness: F) -> Self {
        Self {
            old_tenseness: tenseness,
            new_tenseness: tenseness,
            target_tenseness: tenseness,
        }
    }

    fn update(&mut self, time: F) {
        self.old_tenseness = self.new_tenseness;
        self.new_tenseness =
            self.target_tenseness + 0.1 * simplex1(time * 0.46) + 0.05 * simplex1(time * 0.36);
    }

    fn get(&self, lambda: F) -> F {
        lerp(self.old_tenseness, self.new_tenseness, lambda)
    }
}
