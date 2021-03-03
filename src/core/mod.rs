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
pub mod param2;
pub mod placeholder;
pub mod proc_once;
pub mod proc_once_share;
pub mod ring_buffer_playback;
pub mod ring_buffer_record;
pub mod sine;

use crate::ring_buffer::RingBuffer;
use std::borrow::Borrow;

pub use crate::{Node, ProcContext};

use self::{proc_once_share::ProcOnceShare, ring_buffer_record::RingBufferRecord};

impl<T, A> Borrow<RingBuffer<T>>
    for ProcOnceShare<T, RingBufferRecord<T, A>>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    fn borrow(&self) -> &RingBuffer<T> {
        &self.get_ref().get_ref().borrow()
    }
}
