pub mod accumulator;
pub mod add;
pub mod amp;
pub mod constant;
pub mod controllable;
pub mod envelope;
pub mod map;
pub mod mix;
pub mod param;
pub mod param2;
pub mod proc_once;
pub mod proc_once_share;
pub mod share;
pub mod sine;

use crate::proc_context::ProcContext;

pub trait Node<T: 'static> {
    fn proc(&mut self, ctx: &ProcContext) -> T;
    fn lock(&mut self);
    fn unlock(&mut self);
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
