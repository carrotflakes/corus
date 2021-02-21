use corus::{
    contrib::{
        envelope::AdsrEnvelope,
        fm_synth::FmSynth,
    },
    proc_context::ProcContext,
};

use corus::node::{self};
use node::{
    constant::Constant,
    Node,
};

fn main() {
    let sample_rate = 44100;

    #[rustfmt::skip]
    let mut node = FmSynth::new([
        (Constant::from(1.0), Constant::from(0.0), AdsrEnvelope {a: 0.01, d: 0.5, s: 0.3, r: 0.3}, 1.0, vec![]),
        (Constant::from(1.01), Constant::from(0.0), AdsrEnvelope {a: 0.5, d: 0.5, s: 0.7, r: 0.3}, 2000.0, vec![3]),
        (Constant::from(0.0), Constant::from(4.0), AdsrEnvelope {a: 0.1, d: 0.5, s: 0.3, r: 0.3}, 5.0, vec![3]),
        (Constant::from(1.0), Constant::from(0.0), AdsrEnvelope {a: 0.02, d: 0.5, s: 0.3, r: 0.3}, 1.0, vec![4]),
    ]);
    node.note_on(0.0, 440.0);
    node.note_off(2.0);
    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("fm_synth.wav");
    node.lock();
    for s in pc.into_iter(&mut node).take(sample_rate as usize * 3) {
        writer.write(s.0[0], s.0[0]);
    }
    node.unlock();
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
