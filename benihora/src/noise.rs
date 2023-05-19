use biquad::{Biquad, Coefficients, DirectForm2Transposed, ToHertz};

use crate::F;

pub struct Noise {
    rand: u32,
    filter: DirectForm2Transposed<F>,
}

impl Noise {
    pub fn new(seed: u32, sample_rate: F, frequency: F) -> Self {
        Self {
            rand: seed,
            filter: DirectForm2Transposed::<F>::new(
                Coefficients::<F>::from_params(
                    biquad::Type::BandPass,
                    sample_rate.hz(),
                    frequency.hz(),
                    0.5,
                )
                .unwrap(),
            ),
        }
    }

    pub fn process(&mut self) -> F {
        self.rand = self.rand.overflowing_mul(48271).0 % ((1 << 31) - 1);
        let x = (self.rand << 1) as f64 / std::u32::MAX as f64;
        let x = x * 2.0 - 1.0;
        self.filter.run(x)
    }
}
