use corus::{node::Node, proc_context::ProcContext, signal::C2f32};

pub fn write_to_file<N: Node<C2f32>, DN: AsMut<N>>(
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
    let pc = ProcContext::new(sample_rate as u64);
    let start = std::time::Instant::now();
    node.as_mut().lock();
    for s in pc
        .into_iter(&mut node)
        .take((sample_rate as f64 * len).ceil() as usize)
    {
        writer
            .write_sample((s.0[0] * std::i16::MAX as f32) as i16)
            .unwrap();
        writer
            .write_sample((s.0[1] * std::i16::MAX as f32) as i16)
            .unwrap();
    }
    node.as_mut().unlock();
    writer.finalize().unwrap();
    println!("{:?} elapsed", start.elapsed());
}

#[allow(dead_code)]
fn main() {}
