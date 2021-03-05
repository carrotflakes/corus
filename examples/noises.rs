mod write_to_file;

use corus::{
    contrib::{
        buffer_playback::BufferPlayback, fn_processor::FnProcessor, rand::Rand, render_to_buffer,
    },
    time::Second,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut rand = Rand::new(1);
    let mut noise = FnProcessor::new(move || rand.next_f64() * 2.0 - 1.0);
    let buf1 = render_to_buffer(SAMPLE_RATE as u64, Second(1.0), &mut noise);

    let mut rand = Rand::new(1);
    let mut noise = FnProcessor::new(move || (rand.next_f64() * std::f64::consts::PI * 2.0).sin());
    let buf2 = render_to_buffer(SAMPLE_RATE as u64, Second(1.0), &mut noise);

    let mut rand = Rand::new(1);
    let mut noise = FnProcessor::new(move || {
        (rand.next_f64() + rand.next_f64() + rand.next_f64() + rand.next_f64()) / 2.0 - 1.0
    });
    let buf3 = render_to_buffer(SAMPLE_RATE as u64, Second(1.0), &mut noise);

    let node = BufferPlayback::new(
        buf1.iter()
            .chain(buf2.iter())
            .chain(buf3.iter())
            .copied()
            .collect::<Vec<f64>>(),
    );

    write_to_file::write_to_file("noises.wav", SAMPLE_RATE, 3.0, node, None, None);
}
