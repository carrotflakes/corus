use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Envelope {
    a: f32,
    d: f32,
    s: f32,
    r: f32,
    dur: f32,
    time: f32,
}

impl Envelope {
    pub fn new(a: f32, d: f32, s: f32, r: f32, dur: f32) -> Self {
        Envelope {a, d, s, r, dur, time: 0.0}
    }

    pub fn get(&self, time: f32) -> f32 {
        if time < self.a {
            time / self.a
        } else if time - self.a < self.s {
            1.0 - (1.0 - self.d) * (time - self.a) / self.s
        } else {
            self.d
        }
    }
}

impl Node<f32> for Envelope {
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        self.time += 1.0 / ctx.sample_rate as f32;
        if self.dur + self.s <= self.time {
            0.0
        } else if self.dur <= self.time {
            self.get(self.dur) * (1.0 - (self.time - self.dur) / self.s)
        } else {
            self.get(self.time)
        }
    }
}

#[test]
fn test() {
    let mut env = Envelope {
        a: 0.1,
        d: 0.25,
        s: 0.1,
        r: 0.2,
        dur: 0.5,
        time: 0.0,
    };
    let pc = ProcContext::new(20);
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
    dbg!(env.proc(&pc));
}
