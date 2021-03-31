use crate::EventListener;

use super::{Node, ProcContext};

pub struct Envelope {
    points: Vec<(f64, f64, Curve)>, // length, level, curve
    release_length: f64,
    release_curve: Curve,
    note_on_time: f64,
    note_off_time: f64,
    level_at_note_off_time: f64,
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
        Self {
            points,
            release_length,
            release_curve: Curve::from_level(release_curve_level),
            note_on_time: 0.0,
            note_off_time: 0.0,
            level_at_note_off_time: 0.0,
        }
    }

    #[inline]
    pub fn note_on(&mut self, time: f64) {
        self.note_on_time = time;
        self.note_off_time = std::f64::INFINITY;
    }

    #[inline]
    pub fn note_off(&mut self, time: f64) {
        self.note_off_time = time;
        self.level_at_note_off_time =
            self.compute_level(self.note_off_time - self.note_on_time);
    }

    #[inline]
    pub fn compute_level(&self, mut elapsed: f64) -> f64 {
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

impl Node for Envelope {
    type Output = f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        if ctx.current_time < self.note_on_time
            || self.note_off_time + self.release_length <= ctx.current_time
        {
            0.0
        } else if ctx.current_time < self.note_off_time {
            self.compute_level(ctx.current_time - self.note_on_time)
        } else {
            self.release_curve.compute(
                self.level_at_note_off_time,
                0.0,
                (ctx.current_time - self.note_off_time) / self.release_length,
            )
        }
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
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

pub enum EnvelopeEvent {
    NoteOn,
    NoteOff,
}

impl EventListener<EnvelopeEvent> for Envelope {
    #[inline]
    fn apply_event(&mut self, time: f64, event: &EnvelopeEvent) {
        match event {
            EnvelopeEvent::NoteOn => {
                self.note_on(time);
            }
            EnvelopeEvent::NoteOff => {
                self.note_off(time);
            }
        }
    }
}

#[test]
fn test() {
    let env = Envelope::new(&[(0.1, 1.0, -1.0), (1.0, 0.5, 1.0)], 0.3, 1.0);
    dbg!(env.compute_level(0.0));
    dbg!(env.compute_level(0.1));
    dbg!(env.compute_level(0.5));
    dbg!(env.compute_level(0.9));
    dbg!(env.compute_level(1.0));
    dbg!(env.compute_level(1.1));
}
