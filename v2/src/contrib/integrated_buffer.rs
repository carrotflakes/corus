use std::ops::Range;

use crate::{interpolate_get, signal::Signal};

pub struct IntegratedBuffer<S: Signal> {
    buffer: Vec<S>,
}

impl<S: Signal> IntegratedBuffer<S> {
    pub fn from_slice(slice: &[S]) -> Self {
        let mut buffer = Vec::with_capacity(slice.len());
        let mut sum = S::default();
        for &s in slice {
            sum = sum + s;
            buffer.push(sum);
        }
        Self { buffer }
    }

    pub fn get(&self, range: Range<usize>) -> S {
        let repeat = range.end / self.buffer.len() - range.start / self.buffer.len();
        (self.buffer[range.end % self.buffer.len()] - self.buffer[range.start % self.buffer.len()]
            + *self.buffer.last().unwrap() * S::float_from_f64(repeat as f64))
            / S::float_from_f64(range.len() as f64)
    }

    pub fn get_by_normalized_f64(&self, range: Range<f64>) -> S {
        let repeat = range.end.floor() - range.start.floor();
        (self.buffer[((range.end % 1.0) * self.buffer.len() as f64) as usize]
            - self.buffer[((range.start % 1.0) * self.buffer.len() as f64) as usize]
            + *self.buffer.last().unwrap() * S::float_from_f64(repeat))
            / S::float_from_f64((range.end - range.start) * self.buffer.len() as f64)
    }

    pub fn get_by_normalized_f64_with_linear_interpolation(&self, range: Range<f64>) -> S
    where
        S: Signal<Float = f64>,
    {
        let len = self.buffer.len();
        let getter = |i| {
            self.buffer[i % len]
                + *self.buffer.last().unwrap() * S::float_from_f64((i / len) as f64)
        };
        (interpolate_get(range.end * len as f64, getter)
            - interpolate_get(range.start * len as f64, getter))
            / S::float_from_f64((range.end - range.start) * len as f64)
    }
}
