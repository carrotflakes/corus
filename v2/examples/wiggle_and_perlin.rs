use std::f64::consts::TAU;

use corus_v2::{contrib::wiggle::Wiggle, nodes::phase::Phase, ProcessContext};

fn main() {
    let mut ctx = ProcessContext::new(44100.0);

    let freq = 3.0;
    let mut phase1 = Phase::new();
    let mut phase2 = Phase::new();
    let mut wiggle = Wiggle::<f64>::new(5.0 * freq, 1);

    let name = "wiggle_and_perlin.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 30 {
        let f = wiggle.process(&ctx);
        let p = phase1.process(&ctx, 440.0 + f * 100.0);
        let x = (p * TAU).sin() * 0.1;
        writer
            .write_sample((x * std::i16::MAX as f64) as i16)
            .unwrap();

        let f = simplex1(ctx.current_time() * freq);
        let p = phase2.process(&ctx, 440.0 + f * 100.0);
        let x = (p * TAU).sin() * 0.1;

        writer
            .write_sample((x * std::i16::MAX as f64) as i16)
            .unwrap();

        ctx.next();
    }
    writer.finalize().unwrap();
}

pub fn simplex1(x: f64) -> f64 {
    perlin_noise::perlin_noise([x * 1.2, x * 0.7, 0.0])
}
