use super::{glottis::Glottis, tract::Tract, F};

pub struct Benihora {
    pub glottis: Glottis,
    pub tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    proc_num: usize,
}

impl Benihora {
    pub fn new(proc_num: usize) -> Self {
        Self {
            glottis: Glottis::new(),
            tract: Tract::new(),
            block_time: 0.04,
            block_updated_time: 0.0,
            proc_num,
        }
    }

    pub fn process(&mut self, current_time: F, sample_rate: u64, v: F) -> F {
        if self.block_updated_time + self.block_time <= current_time {
            self.block_updated_time += self.block_time;
            self.glottis.update_block(current_time, self.block_time);
            self.tract.update_block(self.block_time);
        }

        let lambda = (current_time - self.block_updated_time) / self.block_time; // TODO: wanna remove this
        let (glottal_output, turbulence_noise) =
            self.glottis
                .run_step(current_time, 1.0 / sample_rate as f64, lambda, v);
        let glottal_output = glottal_output + v * 1.0e-16; // avoid subnormal
        let mut vocal_out = 0.0;
        for i in 0..self.proc_num {
            let time = current_time + i as f64 / self.proc_num as f64 / sample_rate as f64;
            let (mouth, nose) = self.tract.run_step(
                time,
                glottal_output,
                turbulence_noise,
                (time - self.block_updated_time) / self.block_time,
                1.0 / (sample_rate as usize * self.proc_num) as f64,
            );
            vocal_out += mouth + nose;
        }
        (vocal_out / self.proc_num as f64).into()
    }
}
