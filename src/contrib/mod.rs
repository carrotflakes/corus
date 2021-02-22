pub mod buffer_playback;
pub mod chip;
mod effects;
pub mod envelope;
pub mod event_controll;
pub mod fm_synth;
pub mod fn_processor;
mod oscillators;
mod perlin_noise;
pub mod poly_synth;
pub mod rand;
pub mod rand_fm_synth;
pub mod schroeder;

pub use effects::*;
pub use oscillators::*;
pub use perlin_noise::*;

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

pub fn amp_pan<A, G, P, DA, DG, DP>(
    node: DA,
    gain: DG,
    pan: DP,
) -> Pan<f32, C1f32, C1f32, C2f32, Amp<C1f32, A, G, DA, DG>, P, Amp<C1f32, A, G, DA, DG>, DP>
where
    A: Node<C1f32> + 'static,
    G: Node<C1f32> + 'static,
    P: Node<C1f32> + 'static,
    DA: AsMut<A>,
    DG: AsMut<G>,
    DP: AsMut<P>,
{
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
