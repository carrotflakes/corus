use std::f64::consts::TAU;

use benihora::{
    lerp,
    managed::{Intensity, Loudness, Tenseness},
    simplex1, Benihora,
};

pub struct BenihoraManaged {
    pub sound: bool,
    pub frequency: Frequency,
    tenseness: Tenseness,
    pub intensity: Intensity,
    loudness: Loudness,
    pub benihora: Benihora,
    time_offset: f64,
}

impl BenihoraManaged {
    pub fn new(sound_speed: usize, sample_rate: f64, seed: u32) -> Self {
        Self {
            sound: false,
            frequency: Frequency::new(140.0, 0.005, 6.0),
            tenseness: Tenseness::new(0.6),
            intensity: Intensity::new(0.0),
            loudness: Loudness::new(0.6f64.powf(0.25)),
            benihora: Benihora::new(sound_speed, sample_rate, seed),
            time_offset: seed as f64 * 8.0,
        }
    }

    /// let v = v.clamp(0.0, 1.0);
    /// set_tenseness(1.0 - (v * std::f64::consts::PI * 0.5).cos());
    pub fn set_tenseness(&mut self, tenseness: f64) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness.new_loudness = tenseness.powf(0.25);
    }

    pub fn process(&mut self, current_time: f64) -> f64 {
        if self.benihora.tract.update_timer.overflowed() {
            self.intensity
                .update(self.sound, self.benihora.tract.update_timer.interval);
            self.frequency.update(current_time + self.time_offset);
            self.tenseness.update(current_time + self.time_offset);
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
    old_frequency: f64,
    new_frequency: f64,
    target_frequency: f64,
    smooth_frequency: f64,
    pub pitchbend: f64,

    pub vibrato_amount: f64,
    pub vibrato_frequency: f64,
    pub wobble_amount: f64,
}

impl Frequency {
    pub fn new(frequency: f64, vibrato_amount: f64, vibrato_frequency: f64) -> Self {
        Self {
            old_frequency: frequency,
            new_frequency: frequency,
            target_frequency: frequency,
            smooth_frequency: frequency,
            pitchbend: 1.0,
            vibrato_amount,
            vibrato_frequency,
            wobble_amount: 1.0,
        }
    }

    pub fn set(&mut self, frequency: f64) {
        self.target_frequency = frequency;
    }

    fn update(&mut self, time: f64) {
        let mut vibrato = self.vibrato_amount * (TAU * time * self.vibrato_frequency).sin();
        vibrato +=
            self.wobble_amount * (0.02 * simplex1(time * 4.07) + 0.04 * simplex1(time * 2.15));

        self.smooth_frequency +=
            (self.target_frequency * self.pitchbend - self.smooth_frequency) * 0.9;

        self.old_frequency = self.new_frequency;
        self.new_frequency = self.smooth_frequency * (1.0 + vibrato);
    }

    pub fn get(&self, lambda: f64) -> f64 {
        lerp(self.old_frequency, self.new_frequency, lambda)
    }
}
