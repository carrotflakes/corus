mod write_to_file;

use corus::{
    contrib::sma::Sma,
    core::{
        accumulator::Accumulator,
        biquad_filter::{BiquadFilter, BiquadFilterParams, LowPass},
        map::Map,
        mul::Mul,
        param3::ParamEventScheduleNode,
        sine::Sine,
        var::Var,
    },
    notenum_to_frequency,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let freq = ParamEventScheduleNode::from_value(440.0);
    let freq_ctl = freq.get_scheduler();
    let filter_freq = ParamEventScheduleNode::from_value(440.0);
    let filter_freq_ctl = filter_freq.get_scheduler();

    let filter_freq_lfo = Map::new(Sine::new(Var::new(0.1)), |x| x * 0.4 + 1.0);
    let filter_freq = Mul::new(filter_freq, filter_freq_lfo);
    let freq = Sma::new(freq, 0.05);
    let node = Accumulator::new(freq, 1.0);
    let node = Map::new(node, |x| x - 0.5);
    let node = BiquadFilter::new(
        node,
        BiquadFilterParams::new(LowPass, filter_freq, Var::new(0.0), Var::new(2.0)),
    );
    let node = Map::new(node, |x: f64| x.clamp(-0.9, 0.9));

    {
        let mut freq_ctl = freq_ctl.lock().unwrap();
        let mut filter_freq_ctl = filter_freq_ctl.lock().unwrap();
        let mut f = |time: i32, nn: u8| {
            let time = time as f64 / 5.0;
            freq_ctl.set_value_at_time(time, notenum_to_frequency(nn - 24));
            filter_freq_ctl.cancel_and_hold_at_time(time);
            filter_freq_ctl.set_value_at_time(time, 8000.0);
            filter_freq_ctl.exponential_ramp_to_value_at_time(time + 0.4, 100.0);
        };
        let mut t = 0;
        for i in 0..4 {
            for _ in 0..4 {
                for n in &[60 - i % 2, 67, 69, 67] {
                    f(t, *n);
                    t += 1;
                }
            }
        }
    }
    write_to_file::write_to_file("acid_bass.wav", SAMPLE_RATE, 16.0, node, None, None);
}
