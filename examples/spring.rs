mod write_to_file;

use corus::{
    contrib::spring::{Spring, SpringEvent},
    core::{constant::Constant, param::Param},
    notenum_to_frequency, EventControlInplace, EventPusher,
};

fn main() {
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, notenum_to_frequency(60));
    freq.set_value_at_time(1.0, notenum_to_frequency(62));
    freq.set_value_at_time(2.0, notenum_to_frequency(64));
    freq.set_value_at_time(3.0, notenum_to_frequency(60));
    freq.set_value_at_time(4.0, notenum_to_frequency(64));
    freq.set_value_at_time(5.0, 10.0);
    freq.set_value_at_time(6.0, 44100.0 / 4.0);
    freq.set_value_at_time(7.0, 1.0);
    freq.set_value_at_time(8.0, 44100.0);
    freq.set_value_at_time(10.0, 5.0);
    let mut decay = Param::with_value(0.001);
    decay.set_value_at_time(13.0, 0.01);
    let mut velocity_limit = Param::with_value(100.0);
    velocity_limit.set_value_at_time(13.0, 6.0);
    let mut target = Param::with_value(0.0);
    target.set_value_at_time(11.0, 0.5);
    target.set_value_at_time(12.0, 0.3);
    target.set_value_at_time(13.0, -0.3);
    target.set_value_at_time(14.0, 0.4);
    target.set_value_at_time(15.0, 0.0);
    let spring = Spring::new(freq, decay, velocity_limit, target, 1.0);
    let mut spring = EventControlInplace::new(spring);
    for i in 0..9 {
        spring.push_event(i as f64, SpringEvent::Reset(0.0, 0.01));
    }
    let node = spring;
    write_to_file::write_to_file("spring.wav", 44100, 20.0, node, None, None);
}
