pub mod accumulator;
pub mod add;
pub mod all_pass_filter;
pub mod amp;
pub mod biquad_filter;
pub mod comb_filter;
pub mod controllable;
pub mod first_order_filter;
pub mod impulse;
pub mod map;
pub mod mix;
pub mod mul;
pub mod pan;
pub mod param;
pub mod var;
// pub mod param2;
pub mod param3;
pub mod placeholder;
pub mod proc_once;
pub mod ring_buffer_playback;
pub mod ring_buffer_record;
pub mod share;
pub mod sine;

use crate::ring_buffer::RingBuffer;
use std::borrow::Borrow;

pub use crate::{Node, ProcContext};

use self::{ring_buffer_record::RingBufferRecord, share::Share};

impl<A> Borrow<RingBuffer<A::Output>> for Share<RingBufferRecord<A>>
where
    A: Node,
    A::Output: 'static + Clone + Default,
{
    fn borrow(&self) -> &RingBuffer<A::Output> {
        &self.get_ref().get_ref().borrow()
    }
}
