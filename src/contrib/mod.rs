pub mod benihora;
pub mod buffer_playback;
pub mod bypass_fader;
pub mod chip;
pub mod crossfader;
pub mod delay;
mod effects;
pub mod envelope;
pub mod envelope2;
pub mod envelope3;
pub mod fm_synth;
pub mod fn_processor;
pub mod generic_poly_synth;
mod oscillators;
pub mod parallel_mix;
pub mod parallel_mix2;
mod perlin_noise;
pub mod poly_synth;
pub mod rand;
pub mod rand_fm_synth;
pub mod resample;
pub mod rms;
pub mod schroeder;
pub mod simple_comp;
pub mod sma;
pub mod smooth;
pub mod spring;
pub mod stop_proc;
pub mod triggerable;

pub use effects::*;
pub use oscillators::*;
pub use perlin_noise::*;

use crate::{
    core::{
        amp::Amp,
        controllable::{Controllable, Controller},
        pan::Pan,
        param::Param,
        Node,
    },
    signal::{C1f64, C2f64, Mono},
    time::AsSample,
    ProcContext,
};

pub fn amp_pan<A, G, P>(node: A, gain: G, pan: P) -> Pan<C2f64, Amp<A, G>, P>
where
    A: Node<Output = C1f64>,
    G: Node<Output = C1f64>,
    P: Node<Output = C1f64>,
{
    Pan::new(Amp::new(node, gain), pan)
}

pub fn controllable_param<T: Mono<Float = f64>>(
    initial_value: f64,
) -> (Controllable<Param<T>>, Controller<Param<T>>) {
    let c = Controllable::new(Param::new());
    let mut ctrl = c.controller();
    ctrl.lock().set_value_at_time(0.0, initial_value);
    (c, ctrl)
}

pub fn render_to_buffer<N: Node, S: AsSample>(
    sample_rate: u64,
    length: S,
    node: &mut N,
) -> Vec<N::Output> {
    ProcContext::new(sample_rate).lock(node, length).collect()
}
