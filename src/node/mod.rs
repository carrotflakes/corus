pub mod sine;
pub mod constant;
pub mod amp;
pub mod envelope;
pub mod add;
pub mod param;

pub struct ProcContext {
    pub sample_rate: u64,
    pub time: f64,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            time: 0.0,
        }
    }

    pub fn sample<T: 'static>(&mut self, node: &mut Box<dyn Node<T>>) -> T {
        let r = node.proc(self);
        self.time += 1.0 / self.sample_rate as f64;
        r
    }
}

pub trait Node<T: 'static> {
    fn proc(&mut self, ctx: &ProcContext) -> T;
}

#[test]
fn test() {
    let sine = sine::Sine::new(Box::new(constant::Constant::new(44.0)));
    let mut amp = amp::Amp::new(Box::new(sine), Box::new(constant::Constant::new(0.9)));
    let ctx = ProcContext::new(44100);
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
}
