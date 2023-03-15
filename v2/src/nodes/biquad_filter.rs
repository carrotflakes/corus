use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

use crate::{
    signal::{Mono, Signal, Stereo},
    ProccessContext,
};

pub struct BiquadFilter<const N: usize, S: Signal> {
    biquad: [DirectForm1<S::Float>; N],
}

impl<const N: usize, S: Signal<Float = f64>> BiquadFilter<N, S> {
    pub fn new() -> Self {
        assert_eq!(N, S::CHANNEL);

        let coeffs = Coefficients::<f64>::from_params(
            Type::LowPass,
            44100.0.hz(),
            1.0.khz(),
            Q_BUTTERWORTH_F64,
        )
        .unwrap();
        Self {
            biquad: [DirectForm1::<f64>::new(coeffs); N],
        }
    }
}

impl<S: Signal<Float = f64> + Mono> BiquadFilter<1, S> {
    pub fn process(&mut self, ctx: &ProccessContext, freq: f64, q: f64, x: S) -> S {
        let coeff =
            Coefficients::<f64>::from_params(Type::LowPass, ctx.sample_rate().hz(), freq.hz(), q)
                .unwrap();
        self.biquad[0].update_coefficients(coeff);
        S::from(self.biquad[0].run(x.get_m()))
    }
}

impl<S: Signal<Float = f64> + Stereo> BiquadFilter<2, S> {
    pub fn process(&mut self, ctx: &ProccessContext, freq: f64, q: f64, x: S) -> S {
        let coeff =
            Coefficients::<f64>::from_params(Type::LowPass, ctx.sample_rate().hz(), freq.hz(), q)
                .unwrap();
        self.biquad[0].update_coefficients(coeff.clone());
        self.biquad[1].update_coefficients(coeff);
        let l = self.biquad[0].run(x.get_l());
        let r = self.biquad[1].run(x.get_r());
        S::from_lr(l, r)
    }
}
