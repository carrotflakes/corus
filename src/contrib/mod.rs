pub mod chip;
mod effects;
pub mod envelope;
pub mod event_controll;
pub mod fm_synth;
mod oscillators;
pub mod poly_synth;
pub mod rand;
pub mod rand_fm_synth;

pub use effects::*;
pub use oscillators::*;

use crate::{
    node::{
        amp::Amp,
        controllable::{Controllable, Controller},
        pan::Pan,
        param::Param,
        Node,
    },
    signal::{C1f32, C2f32},
};

pub fn amp_pan<A: Node<C1f32> + 'static, G: Node<C1f32> + 'static, P: Node<C1f32> + 'static>(
    node: impl AsMut<A>,
    gain: impl AsMut<G>,
    pan: impl AsMut<P>,
) -> impl Node<C2f32> {
    Pan::new(Amp::new(node, gain), pan)
}

pub fn controllable_param(
    initial_value: f32,
) -> (Controllable<C1f32, Param>, Controller<C1f32, Param>) {
    let c = Controllable::new(Param::new());
    let mut ctrl = c.controller();
    ctrl.lock().set_value_at_time(0.0, initial_value);
    (c, ctrl)
}
