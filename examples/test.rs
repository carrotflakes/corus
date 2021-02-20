use corus::{node::map::Map, proc_context::ProcContext, signal::C1f32};

use corus::node::{self};
use node::{accumulator::Accumulator, constant::Constant};

fn main() {
    let sample_rate = 44100;

    let node = Map::new(Accumulator::new(Constant::from(440.0), C1f32::from(1.0)), |v| v + C1f32::from(-0.5));
    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("test.wav");
    for s in pc.into_iter(node).take(sample_rate as usize * 3) {
        writer.write(s.0[0], s.0[0]);
    }
    writer.finish();
}


pub struct Writer(hound::WavWriter<std::io::BufWriter<std::fs::File>>);

impl Writer {
    pub fn new(name: &str) -> Self {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        Writer(hound::WavWriter::create(name, spec).unwrap())
    }

    pub fn write(&mut self, sample1: f32, sample2: f32) {
        self.0
            .write_sample((sample1 * std::i16::MAX as f32) as i16)
            .unwrap();
        self.0
            .write_sample((sample2 * std::i16::MAX as f32) as i16)
            .unwrap();
    }

    pub fn finish(self) {
        self.0.finalize().unwrap();
    }
}
