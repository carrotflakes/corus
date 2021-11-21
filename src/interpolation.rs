use crate::signal::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    NearestNeighbor,
    Linear,
}

impl Interpolation {
    pub fn tap<S: Signal>(self, buffer: &[S], x: f64) -> S {
        let get = |x: usize| {
            if buffer.len() <= x {
                return S::default();
            }
            buffer[x].clone()
        };

        match self {
            Interpolation::NearestNeighbor => {
                let x_floor = x.floor() as usize;
                get(x_floor)
            }
            Interpolation::Linear => {
                let x_floor = x.floor() as usize;
                get(x_floor).lerp(&get(x_floor + 1), S::Float::from(x.fract()))
            }
        }
    }
}
