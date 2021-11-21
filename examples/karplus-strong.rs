mod write_to_file;

use corus::{
    contrib::{amp_pan, chip::Noise},
    core::{
        add::Add, amp::Amp, first_order_filter::FirstOrderFilter, param::Param,
        placeholder::Placeholder, ring_buffer_playback::RingBufferPlayback,
        ring_buffer_record::RingBufferRecord, share::Share, var::Var, Node,
    },
    notenum_to_frequency,
    signal::C1f64,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let node = Noise::new();
    let mut env = Param::new();
    env.set_value_at_time(0.0, 1.0);
    env.set_value_at_time(0.001, 0.0);
    env.set_value_at_time(0.5, 1.0);
    env.set_value_at_time(0.501, 0.0);
    env.set_value_at_time(1.0, 1.0);
    env.set_value_at_time(1.001, 0.0);
    env.set_value_at_time(1.5, 1.0);
    env.set_value_at_time(1.501, 0.0);
    let node = Amp::new(node, env);

    let mut delay = Param::new();
    delay.set_value_at_time(
        0.0,
        1.0 / notenum_to_frequency(60) - 0.5 / SAMPLE_RATE as f64,
    );
    delay.set_value_at_time(
        0.5,
        1.0 / notenum_to_frequency(64) - 0.5 / SAMPLE_RATE as f64,
    );
    delay.set_value_at_time(
        1.0,
        1.0 / notenum_to_frequency(67) - 0.5 / SAMPLE_RATE as f64,
    );
    delay.set_value_at_time(
        1.5,
        1.0 / notenum_to_frequency(72) - 0.5 / SAMPLE_RATE as f64,
    );

    let node = {
        let mut p = Placeholder::new(None);
        let mut ps = p.setter();
        let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
        unsafe {
            ps.set(Box::new(Add::new(
                node,
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
    let node = amp_pan(node, Var::from(0.5), Var::from(0.0));

    write_to_file::write_to_file("karplus-strong.wav", SAMPLE_RATE, 10.0, node, None, None);
}
