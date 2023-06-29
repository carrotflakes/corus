use std::f64::consts::TAU;

use benihora::{
    lerp,
    managed::{Loudness, Tenseness},
    simplex1, Benihora, IntervalTimer,
};

pub struct BenihoraManaged {
    pub sound: bool,
    pub frequency: Frequency,
    tenseness: Tenseness,
    pub intensity: Intensity,
    loudness: Loudness,
    pub benihora: Benihora,
    time_offset: f64,
    update_timer: IntervalTimer,
    dtime: f64,
}

impl BenihoraManaged {
    pub fn new(sound_speed: f64, sample_rate: f64, over_sample: f64, seed: u32) -> Self {
        Self {
            sound: false,
            frequency: Frequency::new(140.0, 0.005, 6.0, sample_rate),
            tenseness: Tenseness::new(0.6),
            intensity: Intensity::new(sample_rate),
            loudness: Loudness::new(0.6f64.powf(0.25)),
            benihora: Benihora::new(sound_speed, sample_rate, over_sample, seed),
            time_offset: seed as f64 * 8.0,
            update_timer: IntervalTimer::new_overflowed(0.04),
            dtime: 1.0 / sample_rate,
        }
    }

    pub fn set_tenseness(&mut self, tenseness: f64) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness.new_loudness = tenseness.powf(0.25);
    }

    pub fn process(&mut self, current_time: f64) -> f64 {
        if self.update_timer.overflowed() {
            self.frequency.update(current_time + self.time_offset);
            self.tenseness.update(current_time + self.time_offset);
            self.loudness.update();
        }
        let lambda = self.update_timer.progress();
        self.update_timer.update(self.dtime);

        let intensity = self.intensity.process(if self.sound { 1.0 } else { 0.0 });
        let frequency = self.frequency.get(lambda);
        let tenseness = self.tenseness.get(lambda);
        let loudness = self.loudness.get(lambda);
        self.benihora
            .process(current_time, frequency, tenseness, intensity, loudness)
    }
}

pub struct Frequency {
    value: f64,
    pub pid: PIDController,
    old_vibrate: f64,
    new_vibrate: f64,
    target_frequency: f64,
    pub pitchbend: f64,

    pub vibrato_amount: f64,
    pub vibrato_frequency: f64,
    pub wobble_amount: f64,
}

impl Frequency {
    pub fn new(
        frequency: f64,
        vibrato_amount: f64,
        vibrato_frequency: f64,
        sample_rate: f64,
    ) -> Self {
        Self {
            value: frequency,
            pid: PIDController::new(50.0, 20.0, 0.3, sample_rate),
            old_vibrate: 1.0,
            new_vibrate: 1.0,
            target_frequency: frequency,
            pitchbend: 1.0,
            vibrato_amount,
            vibrato_frequency,
            wobble_amount: 1.0,
        }
    }

    pub fn set(&mut self, frequency: f64, reset: bool) {
        self.target_frequency = frequency;
        if reset {
            self.value = frequency;
        }
    }

    fn update(&mut self, time: f64) {
        let mut vibrato = self.vibrato_amount * (TAU * time * self.vibrato_frequency).sin();
        vibrato +=
            self.wobble_amount * (0.02 * simplex1(time * 4.07) + 0.04 * simplex1(time * 2.15));

        self.old_vibrate = self.new_vibrate;
        self.new_vibrate = 1.0 + vibrato;
    }

    pub fn get(&mut self, lambda: f64) -> f64 {
        let vibrate = lerp(self.old_vibrate, self.new_vibrate, lambda);
        let target_frequency = self.target_frequency * vibrate * self.pitchbend;
        // self.value *= self.pid.process(target_frequency / self.value - 1.0) * self.pid.dtime + 1.0;
        self.value += self.pid.process(target_frequency - self.value) * self.pid.dtime;
        self.value = self.value.clamp(10.0, 10000.0);
        self.value
    }
}

pub struct Intensity {
    value: f64,
    pub pid: PIDController,
}

impl Intensity {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            value: 0.0,
            pid: PIDController::new(10.0, 100.0, 0.0, sample_rate), // recomend kd = 0.0
        }
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn process(&mut self, target: f64) -> f64 {
        self.value += (self.pid.process(target - self.value) - 1.0) * self.pid.dtime;
        self.value = self.value.max(0.0);
        self.value
    }
}

pub struct PIDController {
    sample_rate: f64,
    dtime: f64,
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub integral: f64,
    pub last: f64,
}

impl PIDController {
    pub fn new(kp: f64, ki: f64, kd: f64, sample_rate: f64) -> Self {
        Self {
            sample_rate,
            dtime: 1.0 / sample_rate,
            kp,
            ki,
            kd,
            integral: 0.0,
            last: 0.0,
        }
    }

    pub fn process(&mut self, x: f64) -> f64 {
        let d = (x - self.last) * self.sample_rate;
        self.integral = self.integral + x * self.dtime;
        let y = x * self.kp + self.integral * self.ki + d * self.kd;
        self.last = x;
        y
    }
}
