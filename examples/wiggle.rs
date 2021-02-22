mod write_to_file;

use corus::{
    contrib::{fn_processor::FnProcessor, perlin_noise},
    node::{add::Add, map::Map, pan::Pan, proc_once_share::ProcOnceShare},
    signal::C1f32,
};

use corus::node::{self};
use node::{accumulator::Accumulator, constant::Constant};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let osc = ProcOnceShare::new(FnProcessor::new({
        let mut i = 0.0;
        move || {
            i += 1.0 / SAMPLE_RATE as f64;
            C1f32::from(perlin_noise(i, i * 0.2, 0.0) as f32 * 40.0)
        }
    }));

    let acc = Accumulator::new(Add::new(Constant::from(440.0), osc.clone()), 1.0.into());
    let node = Map::new(acc, |x| C1f32::from((x.0[0] + -0.5) * 0.5));
    let node = Pan::new(node, Constant::from(0.0));

    write_to_file::write_to_file("wiggle.wav", SAMPLE_RATE, 10.0, node);
}
