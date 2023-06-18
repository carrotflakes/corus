use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F64};

use crate::{
    signal::{Mono, Signal, Stereo},
    ProcessContext,
};

pub type FilterType = Type<f64>;

pub struct BiquadFilter<const N: usize, S: Signal<Float = f64>> {
    biquad: [DirectForm1<S::Float>; N],
}

impl<const N: usize, S: Signal<Float = f64>> BiquadFilter<N, S> {
    pub fn new() -> Self {
        assert_eq!(N, S::CHANNEL);

        let coeffs = Coefficients::<f64>::from_params(
            Type::LowPass,
            48000.0.hz(),
            1.0.khz(),
            Q_BUTTERWORTH_F64,
        )
        .unwrap();
        Self {
            biquad: [DirectForm1::<f64>::new(coeffs); N],
        }
    }

    pub fn update_coefficients(
        &mut self,
        ctx: &ProcessContext,
        r#type: Type<f64>,
        freq: f64,
        q: f64,
    ) {
        let coeffs =
            Coefficients::<f64>::from_params(r#type, ctx.sample_rate().hz(), freq.hz(), q).unwrap();
        for biquad in self.biquad.iter_mut() {
            biquad.update_coefficients(coeffs.clone());
        }
    }
}

impl<S: Signal<Float = f64> + Mono> BiquadFilter<1, S> {
    pub fn process(&mut self, x: S) -> S {
        S::from(self.biquad[0].run(x.get_m()))
    }
}

impl<S: Signal<Float = f64> + Stereo> BiquadFilter<2, S> {
    pub fn process(&mut self, x: S) -> S {
        let l = self.biquad[0].run(x.get_l());
        let r = self.biquad[1].run(x.get_r());
        S::from_lr(l, r)
    }
}
