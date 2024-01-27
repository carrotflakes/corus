use crate::signal::Signal;

#[derive(Debug, Clone)]
pub enum Algo<S: Signal> {
    UpSample {
        in_per_out: S::Float,
        prev_sample: S,
        next_sample: S,
        next_sample_time: S::Float,
    },
    DownSample {
        in_per_out: S::Float,
        out_per_in: S::Float,
        left_value: S,
        right_value: S,
        time: S::Float,
    },
    Identity,
}

pub struct Resample<S: Signal> {
    pub algo: Algo<S>,
}

impl<S: Signal> Resample<S> {
    pub fn new(input_sample_rate: S::Float, output_sample_rate: S::Float) -> Self {
        Self {
            algo: if input_sample_rate < output_sample_rate {
                Algo::UpSample {
                    in_per_out: input_sample_rate / output_sample_rate,
                    prev_sample: S::default(),
                    next_sample: S::default(),
                    next_sample_time: S::float_from_f64(1.0),
                }
            } else if input_sample_rate > output_sample_rate {
                Algo::DownSample {
                    in_per_out: input_sample_rate / output_sample_rate,
                    out_per_in: output_sample_rate / input_sample_rate,
                    left_value: S::default(),
                    right_value: S::default(),
                    time: S::float_from_f64(0.0),
                }
            } else {
                Algo::Identity
            },
        }
    }

    pub fn delay(&self) -> S::Float {
        match self.algo {
            Algo::UpSample { .. } => todo!(),
            Algo::DownSample { .. } => todo!(),
            Algo::Identity => S::float_from_f64(0.0),
        }
    }

    pub fn process(&mut self, mut x: impl FnMut() -> S) -> S {
        match self.algo {
            Algo::UpSample {
                in_per_out,
                ref mut prev_sample,
                ref mut next_sample,
                ref mut next_sample_time,
            } => {
                *next_sample_time = *next_sample_time + in_per_out;
                while S::float_from_f64(1.0) <= *next_sample_time {
                    *next_sample_time = *next_sample_time - S::float_from_f64(1.0);
                    *prev_sample = *next_sample;
                    *next_sample = x();
                }
                let t = *next_sample_time;
                *prev_sample + (*next_sample - *prev_sample) * t
            }
            Algo::DownSample {
                in_per_out,
                out_per_in,
                ref mut left_value,
                ref mut right_value,
                ref mut time,
            } => {
                *time = *time + in_per_out;
                let y = *left_value;
                *left_value = *right_value;
                while S::float_from_f64(1.0) <= *time {
                    *left_value = *left_value + x();
                    *time = *time - S::float_from_f64(1.0);
                }
                let x = x();
                *left_value = *left_value + x * *time;
                *right_value = x * (S::float_from_f64(1.0) - *time);
                *time = *time - S::float_from_f64(1.0);
                y * out_per_in
            }
            Algo::Identity => x(),
        }
    }
}

#[test]
fn test() {
    let mut resample = Resample::<f64>::new(4.0, 5.0);

    let mut i = 0.0;
    for _ in 0..5 {
        dbg!(resample.process(|| {
            i += 1.0;
            i
        }));
    }

    let mut resample = Resample::<f64>::new(4.0, 3.0);

    let mut i = 0.0;
    for _ in 0..6 {
        dbg!(resample.process(|| {
            i += 1.0;
            i
        }));
    }
}

#[test]
fn test2() {
    let mut resample = Resample::<f64>::new(2.0, 3.0);

    for _ in 0..5 {
        dbg!(resample.process(|| { 5.0 }));
    }

    let mut resample = Resample::<f64>::new(3.0, 2.0);

    for _ in 0..5 {
        dbg!(resample.process(|| { 5.0 }));
    }
}
