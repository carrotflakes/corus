mod write_to_file;

use corus::{
    contrib::{fn_processor::FnProcessor, rand::Rand, spring::Spring},
    core::{mul::Mul, param::Param, param3::ParamEventScheduleNode},
    notenum_to_frequency,
};

fn main() {
    let mut rand = Rand::new(1);
    let noise = FnProcessor::new(move || rand.next_f64() * 2.0 - 1.0);

    let env = ParamEventScheduleNode::from_value(0.2);
    {
        let schedule = env.get_scheduler();
        let mut schedule = schedule.lock().unwrap();
        schedule.set_value_at_time(0.0, 1.0);
        schedule.set_value_at_time(0.01, 0.0);
        schedule.set_value_at_time(1.0, 1.0);
        schedule.set_value_at_time(1.01, 0.0);
        schedule.set_value_at_time(2.0, 1.0);
        schedule.set_value_at_time(2.01, 0.0);
        schedule.set_value_at_time(3.0, 1.0);
        schedule.set_value_at_time(3.01, 0.0);
        schedule.set_value_at_time(4.0, 1.0);
        schedule.set_value_at_time(4.01, 0.0);
        schedule.set_value_at_time(5.0, 0.1);
    }

    let noise = Mul::new(noise, env);

    let mut freq = Param::new();
    freq.set_value_at_time(0.0, notenum_to_frequency(60));
    freq.set_value_at_time(1.0, notenum_to_frequency(62));
    freq.set_value_at_time(2.0, notenum_to_frequency(64));
    freq.set_value_at_time(3.0, notenum_to_frequency(60));
    freq.set_value_at_time(4.0, notenum_to_frequency(64));
    freq.set_value_at_time(5.0, notenum_to_frequency(60));
    freq.set_value_at_time(6.0, notenum_to_frequency(62));
    freq.set_value_at_time(7.0, notenum_to_frequency(64));
    freq.set_value_at_time(8.0, notenum_to_frequency(60));
    freq.set_value_at_time(9.0, notenum_to_frequency(64));
    let decay = Param::with_value(0.001);
    let velocity_limit = Param::with_value(10000.0);

    let spring = Spring::new(freq, decay, velocity_limit, noise, 1.0);
    let node = spring;
    write_to_file::write_to_file("bowed_spring.wav", 44100, 12.0, node, None, None);
}
