use std::sync::Arc;

use crate::{PackedEvent, ProccessContext};

struct EnvelopeInner {
    points: Vec<(f64, f64, Curve)>, // length, level, curve
    release_length: f64,
    release_curve: Curve,
    note_on_time: f64,
    note_off_time: f64,
    level_at_note_off_time: f64,
}

impl EnvelopeInner {
    fn note_on(&mut self, time: f64) {
        self.note_on_time = time;
        self.note_off_time = std::f64::INFINITY;
    }

    fn note_off(&mut self, time: f64) {
        self.note_off_time = time;
        self.level_at_note_off_time = self.compute_level(self.note_off_time - self.note_on_time);
    }

    fn compute_level(&self, mut elapsed: f64) -> f64 {
        let mut last_level = 0.0;
        for (length, level, curve) in self.points.clone() {
            if elapsed < length {
                return curve.compute(last_level, level, elapsed / length);
            }
            last_level = level;
            elapsed -= length;
        }
        last_level
    }
}

pub struct Envelope {
    inner: Arc<EnvelopeInner>,
}

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
            inner: Arc::new(EnvelopeInner {
                points,
                release_length,
                release_curve: Curve::from_level(release_curve_level),
                note_on_time: 0.0,
                note_off_time: 0.0,
                level_at_note_off_time: 0.0,
            }),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext) -> f64 {
        let inner = unsafe { std::mem::transmute::<_, &mut EnvelopeInner>(Arc::as_ptr(&self.inner)) };

        let current_time = ctx.current_time();
        if current_time < inner.note_on_time
            || inner.note_off_time + inner.release_length <= current_time
        {
            0.0
        } else if current_time < inner.note_off_time {
            inner.compute_level(current_time - inner.note_on_time)
        } else {
            inner.release_curve.compute(
                inner.level_at_note_off_time,
                0.0,
                (current_time - inner.note_off_time) / inner.release_length,
            )
        }
    }

    pub fn note_on_event(&self, time: f64) -> PackedEvent {
        let inner_arc = self.inner.clone();
        Box::new(move |_time: f64| {
            let inner =
                unsafe { std::mem::transmute::<_, &mut EnvelopeInner>(Arc::as_ptr(&inner_arc)) };
            inner.note_on(time);
        })
    }

    pub fn note_off_event(&self, time: f64) -> PackedEvent {
        let inner_arc = self.inner.clone();
        Box::new(move |_time: f64| {
            let inner =
                unsafe { std::mem::transmute::<_, &mut EnvelopeInner>(Arc::as_ptr(&inner_arc)) };
            inner.note_off(time);
        })
    }
}

#[derive(Clone)]
struct Curve(f64);

impl Curve {
    fn from_level(level: f64) -> Self {
        Curve((-level).exp())
    }

    fn compute(&self, a: f64, b: f64, r: f64) -> f64 {
        a + (b - a) * r.powf(self.0)
    }
}
