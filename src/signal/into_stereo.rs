use std::f64::consts::FRAC_PI_2;

use super::{C1f64, C2f64, Mono, Signal, Stereo};

pub trait IntoStereo<F>: Signal<Float = F> {
    type Output: Stereo<F>;

    fn into_stereo(&self) -> Self::Output;
    fn into_stereo_with_pan(&self, pan: F) -> Self::Output;
}

impl IntoStereo<f64> for C1f64 {
    type Output = C2f64;

    #[inline]
    fn into_stereo(&self) -> Self::Output {
        C2f64::from_lr(self.get_m(), self.get_m())
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f64) -> Self::Output {
        let pan = pan.clamp(-1.0, 1.0);
        let x = (pan + 1.0) * 0.5 * FRAC_PI_2;
        let gain_l = x.cos();
        let gain_r = x.sin();
        C2f64::from_lr(self.get_m() * gain_l, self.get_m() * gain_r)
    }
}

impl IntoStereo<f64> for C2f64 {
    type Output = C2f64;

    #[inline]
    fn into_stereo(&self) -> Self::Output {
        C2f64::from_lr(self.get_l(), self.get_r())
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f64) -> Self::Output {
        let pan = pan.clamp(-1.0, 1.0);
        if pan <= 0.0 {
            let x = (pan + 1.0) * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            C2f64::from_lr(self.get_l() + self.get_r() * gain_l, self.get_r() * gain_r)
        } else {
            let x = pan * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            C2f64::from_lr(self.get_l() * gain_l, self.get_l() * gain_r + self.get_r())
        }
    }
}

#[test]
fn test() {
    dbg!(C1f64::from(1.0).into_stereo_with_pan(0.0));
    dbg!(C1f64::from(1.0).into_stereo_with_pan(-1.0));
    dbg!(C1f64::from(1.0).into_stereo_with_pan(1.0));
    dbg!(C2f64([1.0, 1.0]).into_stereo_with_pan(0.0));
}
