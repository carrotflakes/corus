mod write_to_file;

use corus::{
    contrib::buffer_playback::BufferPlayback,
    core::{constant::Constant, pan::Pan, sine::Sine},
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("beats.wav".to_string());
    println!("load {:?} ...", &file);
    let buf = write_to_file::read_wav_file(&file);

    let node = BufferPlayback::new(buf);
    let node = Pan::new(node, Sine::new(Constant::from(0.25)));

    let file = format!("{}-autopan.wav", file[..file.len() - 4].to_string());
    write_to_file::write_to_file(file.as_str(), SAMPLE_RATE, 8.0, node, None, None);
    println!("saved {:?}", &file);
}
