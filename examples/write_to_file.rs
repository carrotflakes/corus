use std::{collections::hash_map::DefaultHasher, hash::Hasher, io::Write};

use corus::{EventQueue, Node, ProcContext, signal::{C2f64, IntoStereo, Signal, Stereo}, time::{AsSample, Second}};

pub fn write_to_file<N>(
    name: &str,
    sample_rate: usize,
    len: f64,
    node: N,
    f64hash: Option<u64>,
    i16hash: Option<u64>,
) where
    N: Node + 'static,
    N::Output: Signal<Float = f64> + IntoStereo,
{
    write_to_file_with_event_queue(name, sample_rate, len, node, f64hash, i16hash, EventQueue::new())
}

pub fn write_to_file_with_event_queue<N>(
    name: &str,
    sample_rate: usize,
    len: f64,
    mut node: N,
    f64hash: Option<u64>,
    i16hash: Option<u64>,
    event_queue: EventQueue,
) where
    N: Node + 'static,
    N::Output: Signal<Float = f64> + IntoStereo,
{
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let mut pc = ProcContext::new(sample_rate as u64);
    pc.event_queue = event_queue;
    let mut f64hasher = DefaultHasher::new();
    let mut i16hasher = DefaultHasher::new();
    let start = std::time::Instant::now();
    let mut count = 0;
    for s in pc
        .lock(&mut node, Second(len))
    {
        if count % 10000 == 0 {
            print!("\r{:>4}/{}", count / 10000, Second(len).as_sample(sample_rate as u64) / 10000);
            std::io::stdout().flush().unwrap();
        }
        count += 1;
        let s = s.into_stereo();
        if !(s.get_l() as f64).is_finite() || !(s.get_r() as f64).is_finite() {
            panic!("signal is not finite, l: {:?}, r: {:?}", s.get_l(), s.get_r());
        }
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
    let f64hash_act = f64hasher.finish();
    let i16hash_act = i16hasher.finish();
    if let Some(f64hash) = f64hash {
        println!("f64 hash: {:>16x} (expect {:>16x})", f64hash_act, f64hash);
    } else {
        println!("f64 hash: {:>16x}", f64hash_act);
    }
    if let Some(i16hash) = i16hash {
        println!("i16 hash: {:>16x} (expect {:>16x})", i16hash_act, i16hash);
    } else {
        println!("i16 hash: {:>16x}", i16hash_act);
    }
}

#[allow(dead_code)]
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
