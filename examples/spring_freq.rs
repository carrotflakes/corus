mod write_to_file;

use corus::{
    contrib::spring::Spring,
    core::{param::Param, sine::Sine},
    notenum_to_frequency,
};

fn main() {
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 5.0);
    freq.linear_ramp_to_value_at_time(10.0, 10.0);
    let mut decay = Param::with_value(0.001);
    decay.set_value_at_time(0.0, 0.001);
    let mut velocity_limit = Param::with_value(6000.0);
    velocity_limit.set_value_at_time(10.0, 600.0);
    let mut target = Param::with_value(50.0);
    target.set_value_at_time(1.0, notenum_to_frequency(60));
    target.set_value_at_time(2.0, notenum_to_frequency(62));
    target.set_value_at_time(3.0, notenum_to_frequency(64));
    target.set_value_at_time(4.0, notenum_to_frequency(60));
    target.set_value_at_time(5.0, notenum_to_frequency(67));
    target.set_value_at_time(6.0, notenum_to_frequency(64));
    target.set_value_at_time(7.0, notenum_to_frequency(72));
    target.set_value_at_time(10.0, notenum_to_frequency(60));
    let spring = Spring::new(freq, decay, velocity_limit, target, 44100.0);
    let node = Sine::new(spring);
    write_to_file::write_to_file("spring_freq.wav", 44100, 20.0, node, None, None);
}
