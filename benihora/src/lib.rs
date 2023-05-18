mod glottis;
mod tract;

use std::f64::consts::TAU;

pub use glottis::Glottis;
pub use tract::{Constriction, Mouth, Nose, Tract};

type F = f64;

pub struct Benihora {
    pub frequency: Frequency,
    tenseness: Tenseness,
    pub glottis: Glottis,
    pub tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    proc_num: usize,
}

impl Benihora {
    pub fn new(proc_num: usize) -> Self {
        Self {
            frequency: Frequency::new(140.0, 0.005, 6.0),
            tenseness: Tenseness::new(0.6),
            glottis: Glottis::new(),
            tract: Tract::new(),
            block_time: 0.04,
            block_updated_time: 0.0,
            proc_num,
        }
    }

    /// let v = v.clamp(0.0, 1.0);
    /// set_tenseness(1.0 - (v * std::f64::consts::PI * 0.5).cos());
    pub fn set_tenseness(&mut self, tenseness: F) {
        let tenseness = tenseness.clamp(0.0, 1.0);
        self.tenseness.target_tenseness = tenseness;
        self.glottis.loudness = tenseness.powf(0.25);
    }

    pub fn process(
        &mut self,
        current_time: F,
        sample_rate: u64,
        aspiration_noise: F,
        fricative_noise: F,
    ) -> F {
        if self.block_updated_time + self.block_time <= current_time {
            self.block_updated_time += self.block_time;
            self.frequency.update(current_time);
            self.tenseness.update(current_time);
            self.glottis.update_block(self.block_time);
            self.tract.update_block(self.block_time);
        }

        let lambda = (current_time - self.block_updated_time) / self.block_time; // TODO: wanna remove this
        let frequency = self.frequency.get(lambda);
        let tenseness = self.tenseness.get(lambda);
        let glottal_output = self.glottis.run_step(
            current_time,
            1.0 / sample_rate as f64,
            aspiration_noise,
            frequency,
            tenseness,
        );
        let glottal_output = glottal_output + aspiration_noise * 1.0e-16; // avoid subnormal
        let mut vocal_out = 0.0;
        for i in 0..self.proc_num {
            let time = current_time + i as f64 / self.proc_num as f64 / sample_rate as f64;
            let (mouth, nose) = self.tract.run_step(
                time,
                glottal_output,
                fricative_noise,
                (time - self.block_updated_time) / self.block_time,
                1.0 / (sample_rate as usize * self.proc_num) as f64,
            );
            vocal_out += mouth + nose;
        }
        (vocal_out / self.proc_num as f64).into()
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

        self.smooth_frequency = self.smooth_frequency * 0.5 + self.target_frequency * 0.5;

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
