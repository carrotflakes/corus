mod write_to_file;

use corus::{
    contrib::{fn_processor::FnProcessor, perlin_noise},
    core::{accumulator::Accumulator, constant::Constant},
    core::{add::Add, map::Map, pan::Pan, share::Share},
    signal::{C1f64, Mono},
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let osc = Share::new(FnProcessor::new({
        let mut i = 0.0;
        move || {
            i += 1.0 / SAMPLE_RATE as f64;
            C1f64::from(perlin_noise(i, i * 0.2, 0.0) as f32 * 40.0)
        }
    }));

    let acc = Accumulator::new(
        Add::new(Constant::new(C1f64::from(440.0)), osc.clone()),
        1.0.into(),
    );
    let node = Map::new(acc, |x| C1f64::from((x.get_m() + -0.5) * 0.5));
    let node = Pan::new(node, Constant::from(0.0));

    write_to_file::write_to_file("wiggle.wav", SAMPLE_RATE, 10.0, node, None, None);
}
