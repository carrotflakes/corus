use super::{C1f32, C2f32, Signal};

pub trait IntoStereo<F>: Signal<Float=F> {
    type Output: Signal<Float=F>;

    fn into_stereo(&self) -> Self::Output;
    fn into_stereo_with_pan(&self, pan: f32) -> Self::Output;
}

impl IntoStereo<f32> for C1f32 {
    type Output = C2f32;

    #[inline]
    fn into_stereo(&self) -> Self::Output {
        C2f32([self.0[0], self.0[0]])
    }

    #[inline]
    fn into_stereo_with_pan(&self, pan: f32) -> Self::Output {
        let pan = pan.min(1.0).max(-1.0);
        let x = (pan + 1.0) * 0.5;
        let gain_l = (x * std::f32::consts::FRAC_PI_2).cos();
        let gain_r = (x * std::f32::consts::FRAC_PI_2).sin();
        C2f32([self.0[0] * gain_l, self.0[0] * gain_r])
    }
}
