mod write_to_file;

use corus::{
    contrib::{fitzhugh_nagumo::FitzhughNagumo, fn_processor::FnProcessor, rand::Rand},
    core::{mul::Mul, var::Var},
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut rand = Rand::new(1);
    let noise = FnProcessor::new(move || rand.next_f64() * 0.15 + 0.25);

    let node = FitzhughNagumo::new(Var::from(0.1), noise);
    let node = Mul::new(node, Var::from(0.1));
    write_to_file::write_to_file("fitzhugh_nagumo.wav", SAMPLE_RATE, 3.0, node, None, None);
}
