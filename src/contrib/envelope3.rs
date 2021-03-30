use crate::EventListener;

use super::{Node, ProcContext};

pub struct Envelope {
    points: Vec<(f64, f64, f64)>, // length, level, handle
    release_length: f64,
    release_handle: f64,
    note_on_time: f64,
    note_off_time: f64,
}

impl Envelope {
    pub fn new(points: Vec<(f64, f64, f64)>, release_length: f64, release_handle: f64) -> Self {
        Self {
            points,
            release_length,
            release_handle,
            note_on_time: 0.0,
            note_off_time: 0.0,
        }
    }

    #[inline]
    fn compute_level(&self, mut elapsed: f64) -> f64 {
        let mut last_level = 0.0;
        for (length, level, handle) in self.points.clone() {
            if elapsed < length {
                return bezier2(last_level, handle, level, elapsed / length);
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
        if ctx.current_time < self.note_on_time || self.note_off_time + self.release_length <= ctx.current_time {
            0.0
        } else if ctx.current_time < self.note_off_time {
            self.compute_level(ctx.current_time - self.note_on_time)
        } else {
            let level = self.compute_level(self.note_off_time - self.note_on_time);
            bezier2(
                level,
                self.release_handle * level,
                0.0,
                (ctx.current_time - self.note_off_time) / self.release_length,
            )
        }
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

#[inline]
fn bezier2(a: f64, b: f64, c: f64, r: f64) -> f64 {
    let ir = 1.0 - r;
    a * ir.powi(2) + b * (r * ir * 2.0) + c * r.powi(2)
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
                self.note_on_time = time;
                self.note_off_time = std::f64::INFINITY;
            }
            EnvelopeEvent::NoteOff => {
                self.note_off_time = time;
            }
        }
    }
}
