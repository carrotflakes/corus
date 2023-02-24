mod benihora;
mod glottis;
mod tract;

pub use benihora::{Benihora, BenihoraEvent};

use crate::{
    core::{
        add::Add,
        biquad_filter::{types::BandPass, BiquadFilter, BiquadFilterParams},
        var::Var,
        Node,
    },
    signal::{C1f64, Mono},
};

use super::{fn_processor::FnProcessor, perlin_noise, rand::Rand};

type F = f64;

fn simplex1(x: F) -> F {
    perlin_noise(x * 1.2, -x * 0.7, 0.0) as F
}

pub fn make_noise_node() -> Box<dyn Node<Output = f64> + Send + Sync> {
    let node1 = BiquadFilter::new(
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || C1f64::from_m(rand.next_f64() * 2.0 - 1.0))
        },
        BiquadFilterParams::new(BandPass, Var::from(500.0), Var::from(0.0), Var::from(2.5)),
    ); // q 0.5
    let node2 = BiquadFilter::new(
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || C1f64::from_m(rand.next_f64() * 2.0 - 1.0))
        },
        BiquadFilterParams::new(BandPass, Var::from(1000.0), Var::from(0.0), Var::from(2.5)),
    ); // q 0.5
    Box::new(Add::new(node1, node2))
}
