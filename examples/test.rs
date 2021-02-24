use corus::{node::{map::Map, param::Param}, proc_context::ProcContext, signal::{C1f64, Mono}};

use corus::node::{self};
use node::{
    accumulator::Accumulator,
    biquad_filter::BiquadFilter,
    comb_filter::CombFilter,
    constant::Constant,
    Node,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let node = Map::new(
        Accumulator::new(Constant::from(440.0), C1f64::from(1.0)),
        |v| v + C1f64::from(-0.5),
    );
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 220.0);
    freq.exponential_ramp_to_value_at_time(2.0, 4000.0);
    let node = BiquadFilter::new(
        node::biquad_filter::Peaking,
        node,
        freq,
        Constant::from(10.0),
        Constant::from(10.0),
    );
    let node = CombFilter::new(node, 0.01, 0.9.into());
    write_to_file("test.wav", 3, node);
}

pub fn write_to_file<N: Node<C1f64>, DN: AsMut<N>>(name: &str, len: usize, mut node: DN) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let pc = ProcContext::new(SAMPLE_RATE as u64);
    node.as_mut().lock();
    for s in pc.into_iter(&mut node).take(SAMPLE_RATE as usize * len) {
        writer
            .write_sample((s.get_m() * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((s.get_m() * std::i16::MAX as f64) as i16)
            .unwrap();
    }
    node.as_mut().unlock();
    writer.finalize().unwrap();
}
