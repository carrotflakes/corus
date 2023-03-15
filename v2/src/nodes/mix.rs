use crate::signal::Signal;

pub fn mix<S: Signal>(tracks: &[(S::Float, S)]) -> S {
    tracks
        .iter()
        .fold(S::default(), |x, (gain, y)| x.add(y.mul(S::from(*gain))))
}
