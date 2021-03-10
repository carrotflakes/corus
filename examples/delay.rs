mod write_to_file;

use corus::{
    contrib::delay::Delay,
    core::constant::Constant,
    core::{add::Add, amp::Amp, sine::Sine},
};

fn main() {
    let node = Sine::new(Constant::from(440.0));
    let delay = Amp::new(
        Add::new(Sine::new(Constant::from(4.0)), Constant::from(1.0)),
        Constant::from(400.0),
    );
    let node = Delay::new(
        node,
        delay,
        44100,
        corus::contrib::delay::Interpolation::Bilinear,
    );
    write_to_file::write_to_file("delay.wav", 44100, 3.0, node, None, None);
}
