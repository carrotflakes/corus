use num_traits::{Float, FromPrimitive};

use super::{float_array::FloatArray, Signal};

impl<const N: usize, F: 'static + Default + Float + FromPrimitive + Send + Sync> Signal
    for FloatArray<N, F>
{
    type Float = F;

    const CHANNEL: usize = N;

    #[inline]
    fn map(self, f: impl Fn(Self::Float) -> Self::Float) -> Self {
        self.map(f)
    }

    #[inline]
    fn zip_map(self, other: Self, f: impl Fn(Self::Float, Self::Float) -> Self::Float) -> Self {
        self.zip_map(other, f)
    }

    #[inline]
    fn float_from_f64(x: f64) -> Self::Float {
        Self::Float::from_f64(x).unwrap()
    }
}
