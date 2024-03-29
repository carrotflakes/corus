use crate::ProcessContext;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Envelope {
    pub points: Vec<(f64, f64, Curve)>, // length, level, curve
    pub release_length: f64,
    pub release_curve: Curve,
}

#[derive(Clone)]
pub struct State {
    pub note_on_time: f64,
    pub note_off_time: f64,
    pub level_at_note_off: f64,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Curve(pub f64);

impl Envelope {
    pub fn new(
        points_src: &[(f64, f64, f64)],
        release_length: f64,
        release_curve_level: f64,
    ) -> Self {
        let mut points = Vec::with_capacity(points_src.len());
        for (length, level, curve_level) in points_src {
            assert!(0.0 < *length);
            points.push((*length, *level, Curve::from_level(*curve_level)));
        }
        Envelope {
            points,
            release_length,
            release_curve: Curve::from_level(release_curve_level),
        }
    }

    pub fn compute_level(&self, mut elapsed: f64) -> f64 {
        let mut last_level = 0.0;
        for (length, level, curve) in self.points.iter().cloned() {
            if elapsed < length {
                return curve.compute(last_level, level, elapsed / length);
            }
            last_level = level;
            elapsed -= length;
        }
        last_level
    }

    pub fn compute_release(&self, level_at_note_off: f64, elapsed: f64) -> f64 {
        if elapsed < self.release_length {
            self.release_curve
                .compute(level_at_note_off, 0.0, elapsed / self.release_length)
        } else {
            0.0
        }
    }

    pub fn compute(&self, elapsed: f64, note_off_time: f64) -> f64 {
        if elapsed <= note_off_time {
            self.compute_level(elapsed)
        } else {
            self.compute_release(self.compute_level(note_off_time), elapsed - note_off_time)
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            note_on_time: 0.0,
            note_off_time: 0.0,
            level_at_note_off: 0.0,
        }
    }

    pub fn note_on(&mut self, time: f64) {
        self.note_on_time = time;
        self.note_off_time = std::f64::INFINITY;
    }

    pub fn note_off(&mut self, envelope: &Envelope, time: f64) {
        self.note_off_time = time;
        self.level_at_note_off = envelope.compute_level(self.note_off_time - self.note_on_time);
    }

    pub fn process(&self, envelope: &Envelope, ctx: &ProcessContext) -> f64 {
        let current_time = ctx.current_time();
        if current_time < self.note_on_time
            || self.note_off_time + envelope.release_length <= current_time
        {
            0.0
        } else if current_time < self.note_off_time {
            envelope.compute_level(current_time - self.note_on_time)
        } else {
            envelope.release_curve.compute(
                self.level_at_note_off,
                0.0,
                (current_time - self.note_off_time) / envelope.release_length,
            )
        }
    }
}

impl Curve {
    pub fn to_level(&self) -> f64 {
        -self.0.ln()
    }

    pub fn from_level(level: f64) -> Self {
        Curve((-level).exp())
    }

    pub fn compute(&self, a: f64, b: f64, r: f64) -> f64 {
        a + (b - a) * r.powf(self.0)
    }
}
