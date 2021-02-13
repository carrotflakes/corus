use corus::node::ProcContext;

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
use corus::node::{self, Node};

fn f(frequency: Box<dyn Node<f32>>, gain: f32) -> Box<dyn Node<f32>> {
    let sine = Box::new(node::sine::Sine::new(frequency));
    let env = Box::new(node::envelope::Envelope::new(0.1, 0.25, 0.5, 0.5, 2.0));
    let gain = Box::new(node::constant::Constant::new(gain));
    let env = Box::new(node::amp::Amp::new(gain, env));
    Box::new(node::amp::Amp::new(sine, env))
}

fn main() {
    let sample_rate = 44100;
    let modu = f(Box::new(node::constant::Constant::new(440.1)), 3000.0);
    let node = f(Box::new(node::add::Add::new(Box::new(node::constant::Constant::new(440.0)), modu)), 1.0);
    let mut p = node.procedure();
    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("output.wav");
    for _ in 0..sample_rate * 3 {
        let s = p(&pc);
        writer.write(s, s);
    }
    writer.finish();
}
