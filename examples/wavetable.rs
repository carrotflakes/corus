mod write_to_file;

use corus::core::{
    accumulator::Accumulator,
    biquad_filter::{BiquadFilter, BiquadFilterParams, LowPass},
    var::Var,
    map::Map,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let buf = vec![0.0, 0.2, 0.5, 0.3, 0.4, 0.3, 0.3, 0.1, 0.1, 0.1, -1.0];
    let acc = Accumulator::new(Var::new(-1.0), 1.0);
    let node = Map::new(acc, move |f| {
        buf[((f * buf.len() as f64 * 140.0) as usize).rem_euclid(buf.len())]
    });
    // let node = Smooth::new(node, 0.9);
    let node = BiquadFilter::new(
        node,
        BiquadFilterParams::new(
            LowPass,
            Var::from(500.0),
            Var::from(0.0),
            Var::from(1.0),
        ),
    );

    write_to_file::write_to_file("wavetable.wav", SAMPLE_RATE, 3.0, node, None, None);
}
