use serde::{Deserialize, Serialize};

use super::param_pool::{Consumer, ParamPool};

#[derive(Serialize, Deserialize)]
pub struct ParamF64 {
    pub value: f64,
    pub consumer: Consumer,
    pub voice_consumer: Consumer,
}

#[derive(Serialize, Deserialize)]
pub struct Lfo {
    pub frequency: f64,
    pub amp: f64,
}

#[derive(Clone, Copy)]
pub struct EnvelopeState {
    pub elapsed: f64,
    pub note_off_time: f64,
}

impl ParamF64 {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            consumer: Consumer::new(),
            voice_consumer: Consumer::new(),
        }
    }

    pub fn compute(&self, param_pools: &[&ParamPool]) -> f64 {
        match param_pools {
            [ps1] => self.value + self.consumer.get(ps1),
            [ps1, ps2] => self.value + self.consumer.get(ps1) + self.voice_consumer.get(ps2),
            _ => panic!("Invalid param_pools"),
        }
    }
}

impl Lfo {
    pub fn compute(&self, elapsed: f64) -> f64 {
        (elapsed * self.frequency * std::f64::consts::TAU).sin() * self.amp
    }
}
