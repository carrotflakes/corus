mod write_to_file;

use corus::{
    contrib::{
        amp_pan,
        benihora::{make_noise_node, Benihora},
    },
    node::map::Map,
    signal::C1f32,
};

use corus::node::{self};
use node::constant::Constant;

const SAMPLE_RATE: usize = 44100;

fn main() {
    let node = amp_pan(
        Map::new(Benihora::new(make_noise_node()), |c| C1f32([c.0[0] as f32])),
        Constant::from(1.0),
        Constant::from(0.0),
    );
    write_to_file::write_to_file("benihora.wav", SAMPLE_RATE, 10.0, node);
}
