use corus::{contrib::{
        chip::{Noise, NoiseEvent},
        event_controll::EventControll,
    }, proc_context::ProcContext, signal::{C1f64, Mono}};

use corus::node::{self};
use node::Node;

fn main() {
    let sample_rate = 44100;

    let mut node: EventControll<C1f64, NoiseEvent> = EventControll::new(Noise::new());
    node.push_event(2.0 * 0.0, NoiseEvent::ShortFreq(false));
    node.push_event(2.0 * 0.1, NoiseEvent::OriginalFreq(1, 4));
    node.push_event(2.0 * 0.2, NoiseEvent::OriginalFreq(2, 4));
    node.push_event(2.0 * 0.3, NoiseEvent::OriginalFreq(3, 4));
    node.push_event(2.0 * 0.4, NoiseEvent::OriginalFreq(4, 4));
    node.push_event(2.0 * 0.5, NoiseEvent::OriginalFreq(5, 4));
    node.push_event(2.0 * 0.6, NoiseEvent::OriginalFreq(6, 4));
    node.push_event(2.0 * 0.7, NoiseEvent::OriginalFreq(7, 4));
    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("chip_noise.wav");
    node.lock();
    for s in pc.into_iter(&mut node).take(sample_rate as usize * 3) {
        writer.write(s.get_m(), s.get_m());
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

    pub fn write(&mut self, sample1: f64, sample2: f64) {
        self.0
            .write_sample((sample1 * std::i16::MAX as f64) as i16)
            .unwrap();
        self.0
            .write_sample((sample2 * std::i16::MAX as f64) as i16)
            .unwrap();
    }

    pub fn finish(self) {
        self.0.finalize().unwrap();
    }
}
