pub mod sine;
pub mod constant;
pub mod amp;
pub mod envelope;
pub mod add;

pub struct ProcContext {
    pub sample_rate: u64,
    pub sample: u64,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            sample: 0,
        }
    }
}

pub type Procedure<T> = Box<dyn FnMut(&ProcContext) -> T>;

pub trait Node<T: 'static> {
    fn procedure(&self) -> Procedure<T>;
}

#[test]
fn test() {
    let sine = sine::Sine::new(Box::new(constant::Constant::new(44.0)));
    let amp = amp::Amp::new(Box::new(sine), Box::new(constant::Constant::new(0.9)));
    let mut p = amp.procedure();
    let ctx = ProcContext::new(44100);
    dbg!(p(&ctx));
    dbg!(p(&ctx));
    dbg!(p(&ctx));
    dbg!(p(&ctx));
}
