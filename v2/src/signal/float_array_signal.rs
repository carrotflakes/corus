use num_traits::Float;

use super::{float_array::FloatArray, Signal};

impl<const N: usize, F: 'static + Default + Float + Send + Sync> Signal for FloatArray<N, F> {
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
}
