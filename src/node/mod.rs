pub mod accumulator;
pub mod add;
pub mod all_pass_filter;
pub mod amp;
pub mod biquad_filter;
pub mod comb_filter;
pub mod constant;
pub mod controllable;
pub mod impulse;
pub mod map;
pub mod mix;
pub mod pan;
pub mod param;
pub mod placeholder;
pub mod proc_once;
pub mod proc_once_share;
pub mod ring_buffer_playback;
pub mod ring_buffer_record;
pub mod sine;

use crate::proc_context::ProcContext;

pub trait Node<T: 'static> {
    fn proc(&mut self, ctx: &ProcContext) -> T;
    fn lock(&mut self);
    fn unlock(&mut self);
}

use crate::ring_buffer::RingBuffer;
use std::borrow::Borrow;

use self::{proc_once_share::ProcOnceShare, ring_buffer_record::RingBufferRecord};

impl<T, A, DA> Borrow<RingBuffer<T>> for ProcOnceShare<T, RingBufferRecord<T, A, DA>, RingBufferRecord<T, A, DA>>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.get_ref().get_ref().borrow()
    }
}

#[test]
fn test() {
    let sine = sine::Sine::new(Box::new(constant::Constant::from(44.0)));
    let mut amp = amp::Amp::new(Box::new(sine), Box::new(constant::Constant::from(0.9)));
    let ctx = ProcContext::new(44100);
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
    dbg!(amp.proc(&ctx));
}
