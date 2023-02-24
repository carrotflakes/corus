mod write_to_file;

use corus::{
    contrib::{bypass_fader::bypass_fader, schroeder::schroeder_reverb},
    core::{
        accumulator::Accumulator, biquad_filter::BiquadFilter, comb_filter::CombFilter,
        var::Var,
    },
    core::{
        biquad_filter::BiquadFilterParams, map::Map, param::Param, share::Share,
    },
    signal::C1f64,
};

fn main() {
    let node = Map::new(
        Accumulator::new(Var::new(440.0), C1f64::from(1.0)),
        |v| v + C1f64::from(-0.5),
    );
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 220.0);
    freq.exponential_ramp_to_value_at_time(2.0, 4000.0);
    let node = BiquadFilter::new(
        node,
        BiquadFilterParams::new(
            corus::core::biquad_filter::types::Peaking,
            freq,
            Var::from(10.0f64.powf(10.0 / 40.0)),
            Var::from(10.0),
        ),
    );
    let node = CombFilter::new(node, 0.01, 0.9.into());
    let node = bypass_fader(
        Share::new(node),
        &|node| schroeder_reverb(node),
        Var::from(1.0),
    );
    write_to_file::write_to_file("test.wav", 44100, 3.0, node, None, None);
}
