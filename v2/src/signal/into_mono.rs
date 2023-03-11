use super::{Mono, Signal, Stereo, StereoF64};

pub trait IntoMono: Signal {
    type Output: Mono<Float = Self::Float>;

    fn into_mono(&self) -> <Self as IntoMono>::Output;
}

impl IntoMono for f64 {
    type Output = f64;

    #[inline]
    fn into_mono(&self) -> <Self as IntoMono>::Output {
        f64::from_m(self.get_m())
    }
}

impl IntoMono for StereoF64 {
    type Output = f64;

    #[inline]
    fn into_mono(&self) -> <Self as IntoMono>::Output {
        let l = self.get_l();
        let r = self.get_r();
        f64::from_m(l.powi(2).copysign(l) + r.powi(2).copysign(r))
    }
}

#[test]
fn test() {
    assert!((f64::from(1.0).into_mono() - f64::from(1.0)).abs() < 0.000001);
    assert!((f64::from(-1.0).into_mono() - f64::from(-1.0)).abs() < 0.000001);
    assert!(
        (StereoF64::from([2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / 2.0]).into_mono() - f64::from(1.0))
            .abs()
            < 0.000001
    );
}
