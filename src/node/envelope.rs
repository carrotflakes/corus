use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Envelope {
    a: f32,
    d: f32,
    s: f32,
    r: f32,
    dur: f32,
}

impl Envelope {
    pub fn new(a: f32, d: f32, s: f32, r: f32, dur: f32) -> Self {
        Envelope {a, d, s, r, dur}
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
    fn procedure(&self) -> Box<dyn FnMut(&ProcContext) -> f32> {
        let this = self.clone();
        let mut time = 0.0;
        let last = self.get(this.dur);
        Box::new(move |ctx| {
            time += 1.0 / ctx.sample_rate as f32;
            if this.dur + this.s <= time {
                0.0
            } else if this.dur <= time {
                last * (1.0 - (time - this.dur) / this.s)
            } else {
                this.get(time)
            }
        })
    }
}

#[test]
fn test() {
    let env = Envelope {
        a: 0.1,
        d: 0.25,
        s: 0.1,
        r: 0.2,
        dur: 0.5,
    };
    let mut p = env.procedure();
    let pc = ProcContext::new(20);
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
    dbg!(p(&pc));
}
