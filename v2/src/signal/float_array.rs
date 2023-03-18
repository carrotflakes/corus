use num_traits::{Float, One, Zero};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FloatArray<const N: usize, F: Float>([F; N]);

impl<const N: usize, F: Float> FloatArray<N, F> {
    pub fn nan() -> Self {
        FloatArray::from(F::nan())
    }

    pub fn infinity() -> Self {
        FloatArray::from(F::infinity())
    }

    pub fn neg_infinity() -> Self {
        FloatArray::from(F::neg_infinity())
    }

    pub fn neg_zero() -> Self {
        FloatArray::from(F::neg_zero())
    }

    pub fn min_value() -> Self {
        FloatArray::from(F::min_value())
    }

    pub fn min_positive_value() -> Self {
        FloatArray::from(F::min_positive_value())
    }

    pub fn max_value() -> Self {
        FloatArray::from(F::max_value())
    }

    #[inline]
    pub fn map(self, f: impl Fn(F) -> F) -> Self {
        FloatArray(self.0.map(f))
    }

    #[inline]
    pub fn zip_map(mut self, other: Self, f: impl Fn(F, F) -> F) -> Self {
        for i in 0..N {
            self.0[i] = f(self.0[i], other.0[i]);
        }
        self
    }

    pub fn iter(&self) -> impl Iterator<Item = &F> {
        self.0.iter()
    }
}

impl<const N: usize, F: Float> std::ops::Neg for FloatArray<N, F> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        FloatArray(self.0.map(|x| -x))
    }
}

impl<const N: usize, F: Float> std::ops::Add for FloatArray<N, F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, |a, b| a + b)
    }
}

impl<const N: usize, F: Float> std::ops::Sub for FloatArray<N, F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, |a, b| a - b)
    }
}

impl<const N: usize, F: Float> std::ops::Mul for FloatArray<N, F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, |a, b| a * b)
    }
}

impl<const N: usize, F: Float> std::ops::Div for FloatArray<N, F> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, |a, b| a / b)
    }
}

impl<const N: usize, F: Float> std::ops::Rem for FloatArray<N, F> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        self.zip_map(rhs, |a, b| a % b)
    }
}

impl<const N: usize, F: Float> std::ops::Add<F> for FloatArray<N, F> {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        self.map(|x| x + rhs)
    }
}

impl<const N: usize, F: Float> std::ops::Sub<F> for FloatArray<N, F> {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        self.map(|x| x - rhs)
    }
}

impl<const N: usize, F: Float> std::ops::Mul<F> for FloatArray<N, F> {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        self.map(|x| x * rhs)
    }
}

impl<const N: usize, F: Float> std::ops::Div<F> for FloatArray<N, F> {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        self.map(|x| x / rhs)
    }
}

impl<const N: usize, F: Float> std::ops::Rem<F> for FloatArray<N, F> {
    type Output = Self;

    fn rem(self, rhs: F) -> Self::Output {
        self.map(|x| x % rhs)
    }
}

impl<const N: usize, F: Float> Zero for FloatArray<N, F> {
    fn zero() -> Self {
        FloatArray([F::zero(); N])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.is_zero())
    }
}

impl<const N: usize, F: Float> One for FloatArray<N, F> {
    fn one() -> Self {
        FloatArray([F::one(); N])
    }
}

impl<const N: usize, F: Float> std::ops::Index<usize> for FloatArray<N, F> {
    type Output = F;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize, F: Float> std::ops::IndexMut<usize> for FloatArray<N, F> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize, F: Float> Default for FloatArray<N, F> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const N: usize, F: Float> std::ops::Deref for FloatArray<N, F> {
    type Target = [F; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize, F: Float> std::ops::DerefMut for FloatArray<N, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize, F: Float> From<F> for FloatArray<N, F> {
    fn from(x: F) -> Self {
        FloatArray([x; N])
    }
}

impl<const N: usize, F: Float> From<[F; N]> for FloatArray<N, F> {
    fn from(x: [F; N]) -> Self {
        FloatArray(x)
    }
}
