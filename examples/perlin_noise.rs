mod write_to_file;

use corus::contrib::{fn_processor::FnProcessor, perlin_noise};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let node = FnProcessor::new({
        let mut i = 0.0;
        move || {
            i += 1000.0 / SAMPLE_RATE as f64;
            perlin_noise(i, i * 0.2, 0.0)
        }
    });

    write_to_file::write_to_file("perlin_noise.wav", SAMPLE_RATE, 30.0, node, None, None);
}
