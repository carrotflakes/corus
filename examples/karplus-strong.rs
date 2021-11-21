mod write_to_file;

use corus::{
    contrib::{amp_pan, fn_processor::FnProcessor, rand::Rand},
    core::{
        add::Add, amp::Amp, first_order_filter::FirstOrderFilter, mul::Mul, param::Param,
        placeholder::Placeholder, ring_buffer_playback::RingBufferPlayback,
        ring_buffer_record::RingBufferRecord, share::Share, var::Var, Node,
    },
    notenum_to_frequency,
    signal::C1f64,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut rand = Rand::new(1);
    let noise = FnProcessor::new(move || rand.next_f64() * 2.0 - 1.0);

    let mut env = Param::new();
    env.set_value_at_time(0.0, 1.0);
    env.set_value_at_time(0.01, 0.0);
    env.set_value_at_time(0.5, 1.0);
    env.set_value_at_time(0.51, 0.0);
    env.set_value_at_time(1.0, 1.0);
    env.set_value_at_time(1.01, 0.0);
    env.set_value_at_time(1.5, 1.0);
    env.set_value_at_time(1.51, 0.0);
    env.set_value_at_time(3.0, 0.0);
    env.linear_ramp_to_value_at_time(4.0, 0.1);
    env.linear_ramp_to_value_at_time(7.0, 0.0);
    let noise = Amp::new(noise, env);
    let noise = Share::new(noise);

    let mut delay = Param::new();
    let f = |nn: u8| 1.0 / notenum_to_frequency(nn) - 0.5 / SAMPLE_RATE as f64;
    delay.set_value_at_time(0.0, f(60));
    delay.set_value_at_time(0.5, f(64));
    delay.set_value_at_time(1.0, f(67));
    delay.set_value_at_time(1.5, f(72));
    delay.set_value_at_time(4.5, f(71));
    delay.set_value_at_time(4.75, f(69));
    delay.set_value_at_time(5.0, f(67));
    delay.set_value_at_time(5.25, f(65));
    delay.set_value_at_time(5.5, f(64));
    delay.set_value_at_time(5.75, f(62));
    delay.set_value_at_time(6.0, f(60));

    let node = {
        let mut p = Placeholder::new(None);
        let mut ps = p.setter();
        let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
        unsafe {
            ps.set(Box::new(Add::new(
                noise.clone(),
                Amp::new(
                    FirstOrderFilter::new(
                        RingBufferPlayback::new(delay, buffer.clone()),
                        Var::from(0.7),
                    ),
                    Var::from(0.99),
                ),
            )) as Box<dyn Node<Output = C1f64>>);
        }
        buffer
    };

    // cancel original noise
    let node = Add::new(node, Mul::new(noise, Var::from(-1.0)));

    let node = amp_pan(node, Var::from(0.5), Var::from(0.0));

    write_to_file::write_to_file("karplus-strong.wav", SAMPLE_RATE, 10.0, node, None, None);
}
