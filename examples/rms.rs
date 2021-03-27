mod write_to_file;

use corus::{
    contrib::{buffer_playback::BufferPlayback, rms::Rms},
    core::{mul::Mul, pan::Pan, sine::Sine, var::Var},
    time::Second,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("beats.wav".to_string());
    println!("load {:?} ...", &file);
    let buf = write_to_file::read_wav_file(&file);
    let buf_len = buf.len() as f64 / SAMPLE_RATE as f64;

    let buf = BufferPlayback::new(buf);
    let node = Sine::new(Var::new(440.0));
    let node = Pan::new(node, Var::new(0.0));
    let node = Mul::new(node, Rms::new(buf, Second(0.3)));

    let file = format!("{}-rms.wav", file[..file.len() - 4].to_string());
    write_to_file::write_to_file(file.as_str(), SAMPLE_RATE, buf_len, node, None, None);
    println!("saved {:?}", &file);
}
