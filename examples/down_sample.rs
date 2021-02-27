mod write_to_file;

use corus::{
    contrib::down_sample::DownSample,
    core::{accumulator::Accumulator, map::Map, param::Param},
    signal::C1f64,
};

fn main() {
    let mut freq = Param::with_value(440.0);
    freq.exponential_ramp_to_value_at_time(2.0, 880.0);
    let node = Map::new(Accumulator::new(freq, C1f64::from(1.0)), |v| {
        (v * 2.0 * std::f64::consts::PI).sin()
    });
    let node = DownSample::new(node, 0.0, 4000);
    write_to_file::write_to_file("down_sample.wav", 44100, 3.0, node);
}
