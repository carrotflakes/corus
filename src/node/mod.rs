pub mod add;
pub mod amp;
pub mod constant;
pub mod envelope;
pub mod map;
pub mod mix;
pub mod param;
pub mod proc_once;
pub mod sine;

use crate::proc_context::ProcContext;

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
