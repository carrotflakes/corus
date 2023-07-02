use std::f64::consts::TAU;

use corus_v2::{contrib::wiggle::Wiggle, nodes::phase::Phase, ProcessContext};

fn main() {
    let mut ctx = ProcessContext::new(44100.0);

    let mut phase = Phase::new();
    let mut wiggle = Wiggle::<f64>::new(10.0, 9.0, 1);

    let name = "wiggle.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 10 {
        let f = wiggle.process(&ctx);
        let p = phase.process(&ctx, 440.0 + f * 50.0);
        let x = (p * TAU).sin() * 0.1;
        writer
            .write_sample((x * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((x * std::i16::MAX as f64) as i16)
            .unwrap();

        ctx.next();
    }
    writer.finalize().unwrap();
}
