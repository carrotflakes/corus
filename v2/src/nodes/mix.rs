use crate::signal::SignalExt;

pub fn mix<S: SignalExt>(tracks: &[(S::Float, S)]) -> S {
    tracks.iter().fold(S::default(), |x, (gain, y)| {
        x.add(y.mul(S::from_float(*gain)))
    })
}
