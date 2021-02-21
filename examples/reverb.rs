use corus::{
    contrib::{buffer_playback::BufferPlayback, schroeder::schroeder_reverb},
    node::Node,
    proc_context::ProcContext,
    signal::C1f32,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("beats.wav".to_string());
    println!("load {:?} ...", &file);
    let mut wav = hound::WavReader::open(&file).unwrap();
    let mut samples = wav.samples::<i16>();
    let mut buf = Vec::new();
    while let Some(s) = samples.next() {
        samples.next();
        buf.push(C1f32::from(s.unwrap() as f32 / std::i16::MAX as f32));
    }
    let render_len = (buf.len() as f64 / SAMPLE_RATE as f64 + 0.1).ceil() as usize;
    let node = BufferPlayback::new(buf);
    // let node = Impulse::new(C1f32::from(1.0));
    // let node = CombFilter::new(node, 0.01, 0.99.into());
    // let node = AllPassFilter::new(node, 0.01, 0.99.into());
    // let node = Amp::new(node, Constant::from(0.3));
    let node = schroeder_reverb(node);
    let file = format!("{}-reverbed.wav", file[..file.len() - 4].to_string());
    write_to_file(file.as_str(), render_len, node);
    println!("saved {:?}", &file);
}

pub fn write_to_file<N: Node<C1f32>, DN: AsMut<N>>(name: &str, len: usize, mut node: DN) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let pc = ProcContext::new(SAMPLE_RATE as u64);
    node.as_mut().lock();
    for s in pc.into_iter(&mut node).take(SAMPLE_RATE as usize * len) {
        writer
            .write_sample((s.0[0] * std::i16::MAX as f32) as i16)
            .unwrap();
        writer
            .write_sample((s.0[0] * std::i16::MAX as f32) as i16)
            .unwrap();
    }
    node.as_mut().unlock();
    writer.finalize().unwrap();
}
