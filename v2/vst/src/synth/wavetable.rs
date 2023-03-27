use std::sync::Arc;

use corus_v2::interpolate_get;
use serde::{Deserialize, Serialize};

pub type WT = Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>;

#[derive(Serialize, Deserialize)]
pub struct WavetableSettings {
    seed: u64,
    #[serde(skip)]
    wt_cache: Option<(u64, WT)>,
    custome_wt_tree: Option<wavetables::tree::Tree>,
    #[serde(skip)]
    custome_wt: Option<WT>,
    #[serde(skip)]
    buffer: Option<WT>,
    pub use_buffer: bool,
}

impl WavetableSettings {
    pub fn new(seed: u64) -> Self {
        WavetableSettings {
            seed,
            wt_cache: None,
            custome_wt_tree: None,
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
        self.custome_wt_tree = Some(wt);
        self.custome_wt = None;
        self.buffer = None;
    }

    pub fn is_custom_wavetable(&self) -> bool {
        self.custome_wt_tree.is_some()
    }

    pub fn clear_custom_wavetable(&mut self) {
        self.custome_wt_tree = None;
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
        if let Some(wt) = &self.custome_wt_tree {
            if self.custome_wt.is_none() {
                self.custome_wt = Some(wt.build().into());
            }
            self.custome_wt.clone().unwrap()
        } else {
            if let Some((seed, wt)) = &self.wt_cache {
                if *seed == self.seed {
                    return wt.clone();
                }
            }
            let wt: WT = generate_wavetable(self.seed).into();
            self.wt_cache = Some((self.seed, wt.clone()));
            wt
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
