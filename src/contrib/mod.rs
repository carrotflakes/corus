pub mod benihora;
pub mod buffer_playback;
pub mod bypass_fader;
pub mod chip;
pub mod crossfader;
pub mod down_sample;
mod effects;
pub mod envelope;
pub mod fm_synth;
pub mod fn_processor;
pub mod generic_poly_synth;
mod oscillators;
mod perlin_noise;
pub mod poly_synth;
pub mod rand;
pub mod rand_fm_synth;
pub mod schroeder;
pub mod simple_comp;
pub mod smooth;
pub mod triggerable;

pub use effects::*;
pub use oscillators::*;
pub use perlin_noise::*;

use crate::{ProcContext, core::{
        amp::Amp,
        controllable::{Controllable, Controller},
        pan::Pan,
        param::Param,
        Node,
    }, signal::{C1f64, C2f64, Mono}, time::AsSample};

pub fn amp_pan<A, G, P>(
    node: A,
    gain: G,
    pan: P,
) -> Pan<f64, C1f64, C1f64, C2f64, Amp<C1f64, A, G>, P>
where
    A: Node<C1f64>,
    G: Node<C1f64>,
    P: Node<C1f64>,
{
    Pan::new(Amp::new(node, gain), pan)
}

pub fn controllable_param<T: Mono<f64>>(
    initial_value: f64,
) -> (Controllable<T, Param<f64, T>>, Controller<T, Param<f64, T>>) {
    let c = Controllable::new(Param::new());
    let mut ctrl = c.controller();
    ctrl.lock().set_value_at_time(0.0, initial_value);
    (c, ctrl)
}

pub fn render_to_buffer<T: 'static, N: Node<T>, S: AsSample>(
    sample_rate: u64,
    length: S,
    node: &mut N,
) -> Vec<T> {
    ProcContext::new(sample_rate)
        .lock(node, length)
        .collect()
}
