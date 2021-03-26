use super::{C1f64, C2f64, Mono, Signal, Stereo};

pub trait IntoMono: Signal {
    type Output: Mono<Float = Self::Float>;

    fn into_mono(&self) -> <Self as IntoMono>::Output;
}

impl IntoMono for C1f64 {
    type Output = C1f64;

    #[inline]
    fn into_mono(&self) -> <Self as IntoMono>::Output {
        C1f64::from_m(self.get_m())
    }
}

impl IntoMono for C2f64 {
    type Output = C1f64;

    #[inline]
    fn into_mono(&self) -> <Self as IntoMono>::Output {
        let l = self.get_l();
        let r = self.get_r();
        C1f64::from_m(l.powi(2).copysign(l) + r.powi(2).copysign(r))
    }
}

#[test]
fn test() {
    assert!((C1f64::from(1.0).into_mono() - C1f64::from(1.0)).abs() < 0.000001);
    assert!((C1f64::from(-1.0).into_mono() - C1f64::from(-1.0)).abs() < 0.000001);
    assert!((C2f64::from([2.0f64.sqrt() / 2.0, 2.0f64.sqrt() / 2.0]).into_mono() - C1f64::from(1.0)).abs() < 0.000001);
}
