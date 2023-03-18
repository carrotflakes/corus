use std::sync::Arc;

use corus_v2::interpolate_get;

use super::cache::Cache;

pub type WT = Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>;

pub struct WavetableSettings {
    pub seed: u64,
    pub wt_cache: Cache<u64, WT, fn(u64) -> WT>,
    pub wt_buffer_cache: Cache<u64, WT, fn(u64) -> WT>,
    pub use_buffer: bool,
}

impl WavetableSettings {
    pub fn new(seed: u64) -> Self {
        WavetableSettings {
            seed,
            wt_cache: Cache::new(|seed| generate_wavetable(seed).into()),
            wt_buffer_cache: Cache::new(|seed: u64| {
                let wt = generate_wavetable(seed);
                let buffer: Vec<_> = (0..2048).map(|i| wt(i as f64 / 2048.0)).collect();
                Arc::new(move |x| {
                    interpolate_get(x * buffer.len() as f64, |i| buffer[i % buffer.len()])
                })
            }),
            use_buffer: true,
        }
    }

    pub fn generator(&mut self) -> WT {
        if self.use_buffer {
            self.wt_buffer_cache.get(self.seed).clone()
        } else {
            self.wt_cache.get(self.seed).clone()
        }
    }
}

pub fn generate_wavetable(seed: u64) -> Box<dyn Fn(f64) -> f64 + Send + Sync + 'static> {
    match seed {
        0 => wavetables::tree::Tree::Sin.build(),
        1 => wavetables::tree::Tree::Saw.build(),
        2 => wavetables::tree::Tree::Triangle.build(),
        3 => wavetables::tree::Tree::Square.build(),
        4 => wavetables::tree::Tree::Pulse(wavetables::tree::Value::Constant(3.0 / 4.0)).build(),
        5 => wavetables::tree::Tree::Pulse(wavetables::tree::Value::Constant(7.0 / 8.0)).build(),
        _ => {
            let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
            rand_wt::Config {
                least_depth: 1,
                variable_num: 0,
            }
            .generate(&mut rng)
            .build()
        }
    }
}
