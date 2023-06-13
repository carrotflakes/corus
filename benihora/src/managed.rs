use std::f64::consts::TAU;

use crate::{lerp, simplex1, Benihora, F};

pub struct BenihoraManaged {
    pub sound: bool,
    pub frequency: Frequency,
    tenseness: Tenseness,
    pub intensity: Intensity,
    loudness: Loudness,
    pub benihora: Benihora,
}

impl BenihoraManaged {
    pub fn new(sound_speed: usize, sample_rate: F) -> Self {
        Self {
            sound: false,
            frequency: Frequency::new(140.0, 0.005, 6.0),
            tenseness: Tenseness::new(0.6),
            intensity: Intensity::new(0.0),
            loudness: Loudness::new(0.6f64.powf(0.25)),
            benihora: Benihora::new(sound_speed, sample_rate),
        }
    }

    /// let v = v.clamp(0.0, 1.0);
    /// set_tenseness(1.0 - (v * std::f64::consts::PI * 0.5).cos());
    pub fn set_tenseness(&mut self, tenseness: F) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness.new_loudness = tenseness.powf(0.25);
    }

    pub fn process(&mut self, current_time: F) -> F {
        if self.benihora.tract.update_timer.overflowed() {
            self.intensity
                .update(self.sound, self.benihora.tract.update_timer.interval);
            self.frequency.update(current_time);
            self.tenseness.update(current_time);
            self.loudness.update();
        }

        let lambda = self.benihora.tract.update_timer.progress();
        let intensity = self.intensity.get(lambda);
        let frequency = self.frequency.get(lambda);
        let tenseness = self.tenseness.get(lambda);
        let loudness = self.loudness.get(lambda);
        self.benihora
            .process(current_time, frequency, tenseness, intensity, loudness)
    }
}

pub struct Frequency {
    old_frequency: F,
    new_frequency: F,
    pub target_frequency: F,
    smooth_frequency: F,

    pub vibrato_amount: F,
    pub vibrato_frequency: F,
    pub wobble_amount: F,
}

impl Frequency {
    pub fn new(frequency: F, vibrato_amount: F, vibrato_frequency: F) -> Self {
        Self {
            old_frequency: frequency,
            new_frequency: frequency,
            target_frequency: frequency,
            smooth_frequency: frequency,
            vibrato_amount,
            vibrato_frequency,
            wobble_amount: 1.0,
        }
    }

    pub fn set(&mut self, frequency: F) {
        self.target_frequency = frequency;
    }

    pub fn update(&mut self, time: F) {
        let mut vibrato = self.vibrato_amount * (TAU * time * self.vibrato_frequency).sin();
        vibrato +=
            self.wobble_amount * (0.02 * simplex1(time * 4.07) + 0.04 * simplex1(time * 2.15));

        self.smooth_frequency = (self.smooth_frequency + self.target_frequency) * 0.5;

        self.old_frequency = self.new_frequency;
        self.new_frequency = self.smooth_frequency * (1.0 + vibrato);
    }

    pub fn get(&self, lambda: F) -> F {
        lerp(self.old_frequency, self.new_frequency, lambda)
    }
}

pub struct Tenseness {
    old_tenseness: F,
    new_tenseness: F,
    pub target_tenseness: F,
}

impl Tenseness {
    pub fn new(tenseness: F) -> Self {
        Self {
            old_tenseness: tenseness,
            new_tenseness: tenseness,
            target_tenseness: tenseness,
        }
    }

    pub fn update(&mut self, time: F) {
        self.old_tenseness = self.new_tenseness;
        self.new_tenseness =
            self.target_tenseness + 0.1 * simplex1(time * 0.46) + 0.05 * simplex1(time * 0.36);
        self.new_tenseness = self.new_tenseness.clamp(0.0, 1.0);
    }

    pub fn get(&self, lambda: F) -> F {
        lerp(self.old_tenseness, self.new_tenseness, lambda)
    }
}

pub struct Intensity {
    old_intensity: F,
    new_intensity: F,
    pub up_velocity: F,
    pub down_velocity: F,
}

impl Intensity {
    pub fn new(intensity: F) -> Self {
        Self {
            old_intensity: intensity,
            new_intensity: intensity,
            up_velocity: 3.25,
            down_velocity: 5.0,
        }
    }

    pub fn update(&mut self, sound: bool, interval: f64) {
        self.old_intensity = self.new_intensity;
        if sound {
            self.new_intensity += interval * self.up_velocity;
        } else {
            self.new_intensity -= interval * self.down_velocity;
        }
        self.new_intensity = self.new_intensity.clamp(0.0, 1.0);
    }

    pub fn get(&self, lambda: F) -> F {
        lerp(self.old_intensity, self.new_intensity, lambda)
    }
}

pub struct Loudness {
    old_loudness: F,
    pub new_loudness: F,
}

impl Loudness {
    pub fn new(loudness: F) -> Self {
        Self {
            old_loudness: loudness,
            new_loudness: loudness,
        }
    }

    pub fn update(&mut self) {
        self.old_loudness = self.new_loudness;
    }

    pub fn get(&self, lambda: F) -> F {
        lerp(self.old_loudness, self.new_loudness, lambda)
    }
}
