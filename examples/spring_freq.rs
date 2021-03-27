mod write_to_file;

use corus::{
    contrib::spring::{Spring, SpringEvent},
    core::{add::Add, mul::Mul, var::Var, param::Param, sine::Sine},
    EventControlInplace, EventPusher,
};

fn main() {
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 5.0);
    let mut decay = Param::with_value(0.001);
    decay.set_value_at_time(0.0, 0.001);
    let mut velocity_limit = Param::with_value(6.0);
    velocity_limit.set_value_at_time(13.0, 6.0);
    let mut target = Param::with_value(0.0);
    target.set_value_at_time(1.0, 0.3);
    target.set_value_at_time(2.0, 0.5);
    target.set_value_at_time(3.0, -0.3);
    target.set_value_at_time(4.0, 0.4);
    target.set_value_at_time(5.0, 0.0);
    target.set_value_at_time(6.0, 1.0);
    target.set_value_at_time(7.0, 0.0);
    let spring = Spring::new(freq, decay, velocity_limit, target, 1.0);
    let mut spring = EventControlInplace::new(spring);
    for i in 0..9 {
        spring.push_event(i as f64, SpringEvent::Reset(0.0, 0.01));
    }
    let node = Sine::new(Add::new(
        Var::from(440.0),
        Mul::new(spring, Var::from(200.0)),
    ));
    write_to_file::write_to_file("spring_freq.wav", 44100, 20.0, node, None, None);
}
