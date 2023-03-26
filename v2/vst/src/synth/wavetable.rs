use std::sync::Arc;

use corus_v2::interpolate_get;

use super::cache::Cache;

pub type WT = Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>;

pub struct WavetableSettings {
    seed: u64,
    wt_cache: Cache<u64, WT, fn(u64) -> WT>,
    custome_wt: Option<(wavetables::tree::Tree, WT)>,
    buffer: Option<WT>,
    pub use_buffer: bool,
}

impl WavetableSettings {
    pub fn new(seed: u64) -> Self {
        WavetableSettings {
            seed,
            wt_cache: Cache::new(|seed| generate_wavetable(seed).into()),
            custome_wt: None,
            buffer: None,
            use_buffer: true,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
        self.buffer = None;
    }

    pub fn set_custom_wavetable(&mut self, wt: wavetables::tree::Tree) {
        let built = wt.build();
        self.custome_wt = Some((wt, built.into()));
        self.buffer = None;
    }

    pub fn is_custom_wavetable(&self) -> bool {
        self.custome_wt.is_some()
    }

    pub fn clear_custom_wavetable(&mut self) {
        self.custome_wt = None;
        self.buffer = None;
    }

    pub fn generator(&mut self) -> WT {
        if self.use_buffer {
            if let Some(buffer) = &self.buffer {
                buffer.clone()
            } else {
                let wt = self.wavetable();
                let buffer = {
                    let buffer: Vec<_> = (0..2048).map(|i| wt(i as f64 / 2048.0)).collect();
                    Arc::new(move |x| {
                        interpolate_get(x * buffer.len() as f64, |i| buffer[i % buffer.len()])
                    })
                };
                self.buffer = Some(buffer.clone());
                buffer
            }
        } else {
            self.wavetable()
        }
    }

    pub fn wavetable(&mut self) -> Arc<dyn Fn(f64) -> f64 + Send + Sync> {
        if let Some((_, wt)) = &self.custome_wt {
            wt.clone()
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
