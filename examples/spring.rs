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
    let spring = Spring::new(freq, Constant::new(0.001), 1.0);
    let mut spring = EventControlInplace::new(spring);
    for i in 0..9 {
        spring.push_event(i as f64, SpringEvent::Reset(0.0, 1.0 / 441.0));
    }
    let node = spring;
    write_to_file::write_to_file("spring.wav", 44100, 10.0, node, None, None);
}
