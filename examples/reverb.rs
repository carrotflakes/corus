mod write_to_file;

use corus::{
    contrib::{buffer_playback::BufferPlayback, schroeder::schroeder_reverb},
    signal::C1f64,
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
        buf.push(C1f64::from(s.unwrap() as f64 / std::i16::MAX as f64));
    }
    let render_len = buf.len() as f64 / SAMPLE_RATE as f64 + 0.1;
    let node = BufferPlayback::new(buf);
    // let node = Impulse::new(C1f64::from(1.0));
    // let node = CombFilter::new(node, 0.01, 0.99.into());
    // let node = AllPassFilter::new(node, 0.01, 0.99.into());
    // let node = Amp::new(node, Var::from(0.3));
    let node = schroeder_reverb(node);
    let file = format!("{}-reverbed.wav", file[..file.len() - 4].to_string());

    write_to_file::write_to_file(file.as_str(), 44100, render_len, node, None, None);
    println!("saved {:?}", &file);
}
