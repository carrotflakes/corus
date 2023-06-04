mod benihora;
mod glottis;
mod interval_timer;
mod noise;
mod tract;

use std::f64::consts::TAU;

pub use self::benihora::Benihora;
pub use glottis::Glottis;
pub use interval_timer::IntervalTimer;
pub use tract::{Constriction, Mouth, Nose};

type F = f64;

pub struct BenihoraManaged {
    pub sound: bool,
    pub frequency: Frequency,
    tenseness: Tenseness,
    intensity: F,
    loudness: F,
    pub benihora: benihora::Benihora,
}

impl BenihoraManaged {
    pub fn new(sound_speed: usize, sample_rate: F) -> Self {
        Self {
            sound: true,
            frequency: Frequency::new(140.0, 0.005, 6.0),
            tenseness: Tenseness::new(0.6),
            intensity: 0.0,
            loudness: 1.0,
            benihora: benihora::Benihora::new(sound_speed, sample_rate),
        }
    }

    /// let v = v.clamp(0.0, 1.0);
    /// set_tenseness(1.0 - (v * std::f64::consts::PI * 0.5).cos());
    pub fn set_tenseness(&mut self, tenseness: F) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness = tenseness.powf(0.25);
    }

    pub fn process(&mut self, current_time: F) -> F {
        if self.benihora.tract.update_timer.overflowed() {
            if self.sound {
                self.intensity += self.benihora.tract.update_timer.interval * 3.25;
            } else {
                self.intensity -= self.benihora.tract.update_timer.interval * 5.0;
            }
            self.intensity = self.intensity.clamp(0.0, 1.0);

            self.frequency.update(current_time);
            self.tenseness.update(current_time);
        }

        let lambda = self.benihora.tract.update_timer.progress();
        let frequency = self.frequency.get(lambda);
        let tenseness = self.tenseness.get(lambda);
        self.benihora.process(
            current_time,
            frequency,
            tenseness,
            self.intensity,
            self.loudness,
        )
    }
}

pub fn simplex1(x: F) -> F {
    perlin_noise::perlin_noise([x * 1.2, -x * 0.7, 0.0])
}

#[inline]
pub fn lerp(a: F, b: F, t: F) -> F {
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
