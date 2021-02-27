use std::{collections::hash_map::DefaultHasher, hash::Hasher, io::Write};

use corus::{Node, ProcContext, signal::{C2f64, IntoStereo, Stereo}};

pub fn write_to_file<T: IntoStereo<f64>, N: Node<T> + 'static>(
    name: &str,
    sample_rate: usize,
    len: f64,
    mut node: N,
) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let mut pc = ProcContext::new(sample_rate as u64);
    let mut f64hasher = DefaultHasher::new();
    let mut i16hasher = DefaultHasher::new();
    let start = std::time::Instant::now();
    let len_usize = (sample_rate as f64 * len).ceil() as usize;
    let mut count = 0;
    for s in pc
        .lock(&mut node)
        .take(len_usize)
    {
        if count % 10000 == 0 {
            print!("\r{:>4}/{}", count / 10000, len_usize / 10000);
            std::io::stdout().flush().unwrap();
        }
        count += 1;
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
    println!();
    println!("{:?} elapsed", start.elapsed());
    println!("hash(f64): {:x}", f64hasher.finish());
    println!("hash(i16): {:x}", i16hasher.finish());
}

pub fn read_wav_file(file: &str) -> Vec<C2f64> {
    let mut wav = hound::WavReader::open(&file).unwrap();
    println!("spec: {:?}", wav.spec());
    let mut samples = wav.samples::<i16>();
    let mut buf = Vec::new();
    while let Some(l) = samples.next() {
        let r = samples.next().unwrap();
        buf.push(C2f64::from([
            l.unwrap() as f64 / std::i16::MAX as f64,
            r.unwrap() as f64 / std::i16::MAX as f64,
        ]));
    }
    buf
}

#[allow(dead_code)]
fn main() {}
