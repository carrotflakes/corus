mod benihora;
mod glottis;
mod tract;

pub use benihora::Benihora;

use crate::{
    node::{
        add::Add,
        biquad_filter::{BandPass, BiquadFilter},
        constant::Constant,
        Node,
    },
    signal::C2f32,
};

use super::{fn_processor::FnProcessor, perlin_noise, rand::Rand};

type F = f64;

fn simplex1(x: F) -> F {
    perlin_noise(x * 1.2, -x * 0.7, 0.0) as F
}

pub fn make_noise_node() -> Box<dyn Node<C2f32>> {
    let node1 = BiquadFilter::new(
        BandPass,
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || {
                C2f32([rand.next_f32() * 2.0 - 1.0, rand.next_f32() * 2.0 - 1.0])
            })
        },
        Constant::from(500.0),
        Constant::from(0.0),
        Constant::from(2.5),
    ); // q 0.5
    let node2 = BiquadFilter::new(
        BandPass,
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || {
                C2f32([rand.next_f32() * 2.0 - 1.0, rand.next_f32() * 2.0 - 1.0])
            })
        },
        Constant::from(1000.0),
        Constant::from(0.0),
        Constant::from(2.5),
    ); // q 0.5
    Box::new(Add::new(node1, node2))
}
