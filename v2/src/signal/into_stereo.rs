use std::f64::consts::FRAC_PI_2;

use super::{Mono, Signal, Stereo, StereoF64};

pub trait IntoStereo: Signal {
    type Output: Stereo<Float = Self::Float>;

    fn into_stereo(&self) -> <Self as IntoStereo>::Output;
    fn into_stereo_with_pan(&self, pan: Self::Float) -> <Self as IntoStereo>::Output;
}

impl IntoStereo for f64 {
    type Output = StereoF64;

    #[inline]
    fn into_stereo(&self) -> <Self as IntoStereo>::Output {
        StereoF64::from_lr(self.get_m(), self.get_m())
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f64) -> <Self as IntoStereo>::Output {
        let pan = pan.clamp(-1.0, 1.0);
        let x = (pan + 1.0) * 0.5 * FRAC_PI_2;
        let gain_l = x.cos();
        let gain_r = x.sin();
        StereoF64::from_lr(self.get_m() * gain_l, self.get_m() * gain_r)
    }
}

impl IntoStereo for StereoF64 {
    type Output = StereoF64;

    #[inline]
    fn into_stereo(&self) -> <Self as IntoStereo>::Output {
        StereoF64::from_lr(self.get_l(), self.get_r())
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f64) -> <Self as IntoStereo>::Output {
        let pan = pan.clamp(-1.0, 1.0);
        if pan <= 0.0 {
            let x = (pan + 1.0) * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            StereoF64::from_lr(self.get_l() + self.get_r() * gain_l, self.get_r() * gain_r)
        } else {
            let x = pan * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            StereoF64::from_lr(self.get_l() * gain_l, self.get_l() * gain_r + self.get_r())
        }
    }
}

#[test]
fn test() {
    dbg!(f64::from(1.0).into_stereo_with_pan(0.0));
    dbg!(f64::from(1.0).into_stereo_with_pan(-1.0));
    dbg!(f64::from(1.0).into_stereo_with_pan(1.0));
    dbg!([1.0, 1.0].into_stereo_with_pan(0.0));
}
