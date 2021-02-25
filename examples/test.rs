mod write_to_file;

use corus::{contrib::{bypass_fader::bypass_fader, schroeder::schroeder_reverb}, core::{
        accumulator::Accumulator, biquad_filter::BiquadFilter, comb_filter::CombFilter,
        constant::Constant,
    }, core::{map::Map, param::Param, proc_once_share::ProcOnceShare}, signal::C1f64};

fn main() {
    let node = Map::new(
        Accumulator::new(Constant::from(440.0), C1f64::from(1.0)),
        |v| v + C1f64::from(-0.5),
    );
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 220.0);
    freq.exponential_ramp_to_value_at_time(2.0, 4000.0);
    let node = BiquadFilter::new(
        corus::core::biquad_filter::Peaking,
        node,
        freq,
        Constant::from(10.0),
        Constant::from(10.0),
    );
    let node = CombFilter::new(node, 0.01, 0.9.into());
    let node = bypass_fader(ProcOnceShare::new(node), &|node| schroeder_reverb(node), Constant::from(1.0));
    write_to_file::write_to_file("test.wav", 44100, 3.0, node);
}
