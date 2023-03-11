use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

use crate::ProccessContext;

pub struct BiquadFilter {
    biquad: DirectForm1<f64>,
}

impl BiquadFilter {
    pub fn new() -> Self {
        let coeffs = Coefficients::<f64>::from_params(
            Type::LowPass,
            44100.0.hz(),
            1.0.khz(),
            Q_BUTTERWORTH_F64,
        )
        .unwrap();
        Self {
            biquad: DirectForm1::<f64>::new(coeffs),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, freq: f64, q: f64, x: f64) -> f64 {
        self.biquad.update_coefficients(
            Coefficients::<f64>::from_params(Type::LowPass, ctx.sample_rate().hz(), freq.hz(), q)
                .unwrap(),
        );
        self.biquad.run(x)
    }
}
