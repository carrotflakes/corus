use num_traits::Float;

use crate::ProcessContext;

use super::rand::Rand;

pub struct Wiggle<F: Float> {
    frequency: f64,
    frequency_randomness: f64,
    rand: Rand,
    current_value: F,
    next_value: F,
    dvalue: F,
    current_frequency: f64,
    time: f64,
}

impl<F: Float> Wiggle<F> {
    const MIN_FREQUENCY: f64 = f64::EPSILON;

    pub fn new(frequency: f64, frequency_randomness: f64, seed: u32) -> Self {
        let mut rand = Rand::new(seed);
        let current_frequency = (frequency + (rand.next_f64() * 2.0 - 1.0) * frequency_randomness)
            .max(Self::MIN_FREQUENCY);
        Wiggle {
            frequency,
            frequency_randomness,
            current_value: F::from(rand.next_f64() * 2.0 - 1.0).unwrap(),
            next_value: F::from(rand.next_f64() * 2.0 - 1.0).unwrap(),
            dvalue: F::from(0.0).unwrap(),
            current_frequency,
            time: 1.0 / frequency,
            rand,
        }
    }

    pub fn process(&mut self, ctx: &ProcessContext) -> F {
        self.dvalue = self.dvalue
            + (self.next_value - self.current_value)
                * F::from(ctx.dtime() * ctx.dtime() * self.current_frequency).unwrap();
        self.current_value = self.current_value + self.dvalue;
        self.time -= ctx.dtime();
        if self.time < 0.0 {
            self.current_frequency = (self.frequency
                + (self.rand.next_f64() * 2.0 - 1.0) * self.frequency_randomness)
                .max(Self::MIN_FREQUENCY);
            self.time = 1.0 / self.current_frequency;
            self.next_value = F::from(self.rand.next_f64() * 2.0 - 1.0).unwrap();
        }
        self.current_value
    }
}
