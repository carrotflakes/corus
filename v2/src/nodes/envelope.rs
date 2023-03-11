use crate::ProccessContext;

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
        Envelope {
            points,
            release_length,
            release_curve: Curve::from_level(release_curve_level),
            note_on_time: 0.0,
            note_off_time: 0.0,
            level_at_note_off_time: 0.0,
        }
    }

    pub fn note_on(&mut self, time: f64) {
        self.note_on_time = time;
        self.note_off_time = std::f64::INFINITY;
    }

    pub fn note_off(&mut self, time: f64) {
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

    pub fn process(&mut self, ctx: &ProccessContext) -> f64 {
        let current_time = ctx.current_time();
        if current_time < self.note_on_time
            || self.note_off_time + self.release_length <= current_time
        {
            0.0
        } else if current_time < self.note_off_time {
            self.compute_level(current_time - self.note_on_time)
        } else {
            self.release_curve.compute(
                self.level_at_note_off_time,
                0.0,
                (current_time - self.note_off_time) / self.release_length,
            )
        }
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
