use std::f32::consts::FRAC_PI_2;

use super::{C1f32, C2f32, Signal};

pub trait IntoStereo<F>: Signal<Float = F> {
    type Output: Signal<Float = F>;

    fn into_stereo(&self) -> Self::Output;
    fn into_stereo_with_pan(&self, pan: F) -> Self::Output;
}

impl IntoStereo<f32> for C1f32 {
    type Output = C2f32;

    #[inline]
    fn into_stereo(&self) -> Self::Output {
        C2f32([self.get(0), self.get(0)])
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f32) -> Self::Output {
        let pan = pan.clamp(-1.0, 1.0);
        let x = (pan + 1.0) * 0.5;
        let gain_l = (x * FRAC_PI_2).cos();
        let gain_r = (x * FRAC_PI_2).sin();
        C2f32([self.get(0) * gain_l, self.get(0) * gain_r])
    }
}

impl IntoStereo<f32> for C2f32 {
    type Output = C2f32;

    #[inline]
    fn into_stereo(&self) -> Self::Output {
        C2f32([self.get(0), self.get(1)])
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f32) -> Self::Output {
        let pan = pan.clamp(-1.0, 1.0);
        if pan <= 0.0 {
            let x = (pan + 1.0) * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            C2f32([self.get(0) + self.get(1) * gain_l, self.get(1) * gain_r])
        } else {
            let x = pan * FRAC_PI_2;
            let gain_l = x.cos();
            let gain_r = x.sin();
            C2f32([self.get(0) * gain_l, self.get(0) * gain_r + self.get(1)])
        }
    }
}

#[test]
fn test() {
    dbg!(C2f32([1.0, 1.0]).into_stereo_with_pan(0.0f32));
}
