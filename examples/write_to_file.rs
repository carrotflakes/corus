use corus::{node::Node, proc_context::ProcContext, signal::{Stereo, IntoStereo}};

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
    let start = std::time::Instant::now();
    for s in pc
        .lock(&mut node)
        .take((sample_rate as f64 * len).ceil() as usize)
    {
        let s = s.into_stereo();
        writer
            .write_sample((s.get_l() * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((s.get_r() * std::i16::MAX as f64) as i16)
            .unwrap();
    }
    writer.finalize().unwrap();
    println!("{:?} elapsed", start.elapsed());
}

#[allow(dead_code)]
fn main() {}
