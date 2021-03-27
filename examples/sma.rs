mod write_to_file;

use corus::{
    contrib::sma::Sma,
    core::{param3::ParamEventScheduleNode, sine::Sine},
    notenum_to_frequency,
};

fn main() {
    let freq = ParamEventScheduleNode::new();
    let freq_ctl = freq.get_scheduler();
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(0.0, notenum_to_frequency(60));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(0.5, notenum_to_frequency(67));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(1.0, notenum_to_frequency(64));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(1.5, notenum_to_frequency(72));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(2.0, notenum_to_frequency(60));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(2.1, notenum_to_frequency(72));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(2.2, notenum_to_frequency(60));
    freq_ctl
        .lock()
        .unwrap()
        .set_value_at_time(2.3, notenum_to_frequency(72));
    let freq = Sma::new(freq, 0.1);
    let node = Sine::new(freq);
    write_to_file::write_to_file("sma.wav", 44100, 3.0, node, None, None);
}
