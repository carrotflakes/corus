mod write_to_file;

use corus::core::{accumulator::Accumulator, constant::Constant, map::Map};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("beats.wav".to_string());
    println!("load {:?} ...", &file);
    let buf = write_to_file::read_wav_file(&file);

    let acc = Accumulator::new(Constant::from(-1.0), 100.0);
    let node = Map::new(acc, move |f| {
        buf[((f * SAMPLE_RATE as f64) as usize).rem_euclid(buf.len())]
    });
    let file = format!("{}-reversed.wav", file[..file.len() - 4].to_string());

    write_to_file::write_to_file(file.as_str(), 44100, 5.0, node, None, None);
    println!("saved {:?}", &file);
}
