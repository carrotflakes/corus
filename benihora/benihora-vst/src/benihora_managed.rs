use std::f64::consts::TAU;

use benihora::{
    lerp,
    managed::{Loudness, Tenseness},
    wiggle::Wiggle,
    Benihora, IntervalTimer,
};
use serde::{Deserialize, Serialize};

pub struct BenihoraManaged {
    pub sound: bool,
    pub frequency: Frequency,
    pub tenseness: Tenseness,
    pub intensity: Intensity,
    pub loudness: Loudness,
    pub benihora: Benihora,
    update_timer: IntervalTimer,
    sample_rate: f64,
    dtime: f64,
    pub history: Vec<[f32; 4]>,
    pub history_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Params {
    pub frequency_pid: PIDParam,
    pub intensity_pid: PIDParam,
    pub wobble_amount: f64,
    pub vibrato_amount: f64,
    pub vibrato_frequency: f64,
}

impl Params {
    pub fn new() -> Self {
        Self {
            frequency_pid: PIDParam::new(50.0, 20.0, 0.3),
            intensity_pid: PIDParam::new(10.0, 100.0, 0.0), // recomend kd = 0.0
            wobble_amount: 0.1,
            vibrato_amount: 0.005,
            vibrato_frequency: 6.0,
        }
    }
}

impl BenihoraManaged {
    pub fn new(sound_speed: f64, sample_rate: f64, over_sample: f64, seed: u32) -> Self {
        let interval = 0.04;
        Self {
            sound: false,
            frequency: Frequency::new(interval, seed, 140.0, sample_rate),
            tenseness: Tenseness::new(interval, seed, 0.6),
            intensity: Intensity::new(sample_rate),
            loudness: Loudness::new(0.6f64.powf(0.25)),
            benihora: Benihora::new(sound_speed, sample_rate, over_sample, seed),
            update_timer: IntervalTimer::new_overflowed(interval),
            sample_rate,
            dtime: 1.0 / sample_rate,
            history: Vec::new(),
            history_count: 0,
        }
    }

    pub fn set_tenseness(&mut self, tenseness: f64) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.loudness.target = tenseness.powf(0.25);
    }

    pub fn process(&mut self, params: &Params, current_time: f64) -> f64 {
        if self.update_timer.overflowed() {
            self.frequency.update(
                self.update_timer.interval,
                params.wobble_amount,
                params.vibrato_amount,
                params.vibrato_frequency,
            );
            self.tenseness.update();
        }
        let lambda = self.update_timer.progress();
        self.update_timer.update(self.dtime);

        let intensity = self
            .intensity
            .process(&params.intensity_pid, if self.sound { 1.0 } else { 0.0 });
        let frequency = self.frequency.get(&params.frequency_pid, lambda);
        let tenseness = self.tenseness.get(lambda);
        let loudness = self.loudness.process(self.dtime);

        if self.history_count == 0 {
            self.history_count = self.sample_rate as usize / 50;
            self.history.push([
                frequency as f32,
                intensity as f32,
                tenseness as f32,
                loudness as f32,
            ]);
            if self.history.len() > 1000 {
                self.history.remove(0);
            }
        }
        self.history_count -= 1;

        self.benihora
            .process(current_time, frequency, tenseness, intensity, loudness)
    }
}

pub struct Frequency {
    value: f64,
    pid: PIDController,
    old_vibrate: f64,
    new_vibrate: f64,
    target_frequency: f64,
    pub pitchbend: f64,
    phase: f64,

    wiggles: [Wiggle; 2],
}

impl Frequency {
    pub fn new(dtime: f64, seed: u32, frequency: f64, sample_rate: f64) -> Self {
        Self {
            value: frequency,
            pid: PIDController::new(sample_rate),
            old_vibrate: 1.0,
            new_vibrate: 1.0,
            target_frequency: frequency,
            pitchbend: 1.0,
            phase: (seed as f64 / 10.0) % 1.0,
            wiggles: [
                Wiggle::new(dtime / 4.0, 4.07 * 5.0, seed + 1),
                Wiggle::new(dtime / 4.0, 2.15 * 5.0, seed + 2),
            ],
        }
    }

    pub fn set(&mut self, frequency: f64, reset: bool) {
        self.target_frequency = frequency;
        if reset {
            self.value = frequency;
        }
    }

    fn update(
        &mut self,
        dtime: f64,
        wobble_amount: f64,
        vibrato_amount: f64,
        vibrato_frequency: f64,
    ) {
        let mut vibrato = vibrato_amount * (TAU * self.phase).sin();
        self.phase = (self.phase + dtime * vibrato_frequency) % 1.0;
        vibrato +=
            wobble_amount * (0.01 * self.wiggles[0].process() + 0.02 * self.wiggles[1].process());
        for _ in 0..3 {
            self.wiggles[0].process();
            self.wiggles[1].process();
        }

        self.old_vibrate = self.new_vibrate;
        self.new_vibrate = 1.0 + vibrato;
    }

    pub fn get(&mut self, pid: &PIDParam, lambda: f64) -> f64 {
        let vibrate = lerp(self.old_vibrate, self.new_vibrate, lambda);
        let target_frequency = self.target_frequency * vibrate * self.pitchbend;
        // self.value *= self.pid.process(target_frequency / self.value - 1.0) + 1.0;
        self.value += self.pid.process(pid, target_frequency - self.value);
        self.value = self.value.clamp(10.0, 10000.0);
        self.value
    }
}

pub struct Intensity {
    value: f64,
    bias: f64,
    pid: PIDController,
}

impl Intensity {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            value: 0.0,
            bias: -1.0,
            pid: PIDController::new(sample_rate),
        }
    }

    pub fn get(&self) -> f64 {
        self.value
    }

    pub fn process(&mut self, pid: &PIDParam, target: f64) -> f64 {
        self.value += self.pid.process(pid, target - self.value) + self.bias * self.pid.dtime;
        self.value = self.value.max(0.0);
        self.value
    }
}

pub struct PIDController {
    dtime: f64,
    integral: f64,
    last: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PIDParam {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

impl PIDParam {
    pub fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self { kp, ki, kd }
    }
}

impl PIDController {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            dtime: 1.0 / sample_rate,
            integral: 0.0,
            last: 0.0,
        }
    }

    pub fn process(&mut self, pid: &PIDParam, x: f64) -> f64 {
        let d = x - self.last;
        self.integral = self.integral + x * self.dtime;
        let y = (x * pid.kp + self.integral * pid.ki) * self.dtime + d * pid.kd;
        self.last = x;
        y
    }
}
