use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use corus::{Node, ProcContext, signal::{IntoStereo, Stereo}};

pub fn write_to_file<T: IntoStereo<f64>, N: Node<T>, DN: AsMut<N>>(
    name: &str,
    sample_rate: usize,
    len: f64,
    mut node: DN,
) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let mut pc = ProcContext::new(sample_rate as u64);
    let mut f64hasher = DefaultHasher::new();
    let mut i16hasher = DefaultHasher::new();
    let start = std::time::Instant::now();
    for s in pc
        .lock(&mut node)
        .take((sample_rate as f64 * len).ceil() as usize)
    {
        let s = s.into_stereo();
        let l = (s.get_l() * std::i16::MAX as f64) as i16;
        let r = (s.get_r() * std::i16::MAX as f64) as i16;
        f64hasher.write(&s.get_l().to_le_bytes());
        f64hasher.write(&s.get_r().to_le_bytes());
        i16hasher.write_i16(l);
        i16hasher.write_i16(r);
        writer
            .write_sample(l)
            .unwrap();
        writer
            .write_sample(r)
            .unwrap();
    }
    writer.finalize().unwrap();
    println!("{:?} elapsed", start.elapsed());
    println!("hash(f64): {:x}", f64hasher.finish());
    println!("hash(i16): {:x}", i16hasher.finish());
}

#[allow(dead_code)]
fn main() {}
