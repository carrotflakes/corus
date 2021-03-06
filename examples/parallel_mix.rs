mod write_to_file;

use std::sync::{Arc, Mutex};

use corus::{Node, contrib::{bypass_fader::bypass_fader, parallel_mix::ParallelMix, schroeder::schroeder_reverb}, core::{
        accumulator::Accumulator, biquad_filter::BiquadFilter, comb_filter::CombFilter,
        constant::Constant,
    }, core::{amp::Amp, biquad_filter::BiquadFilterParams, map::Map, param::Param, share::Share, sine::Sine}, signal::C1f64};

fn main() {
    let node = Map::new(
        Accumulator::new(Constant::from(440.0), C1f64::from(1.0)),
        |v| v + C1f64::from(-0.5),
    );
    let mut freq = Param::new();
    freq.set_value_at_time(0.0, 220.0);
    freq.exponential_ramp_to_value_at_time(2.0, 4000.0);
    let node = BiquadFilter::new(
        node,
        BiquadFilterParams::new(
            corus::core::biquad_filter::Peaking,
            freq,
            Constant::from(10.0),
            Constant::from(10.0),
        ),
    );
    let node = CombFilter::new(node, 0.01, 0.9.into());
    let node = bypass_fader(
        Share::new(node),
        &|node| schroeder_reverb(node),
        Constant::from(1.0),
    );
    let node = Arc::new(Mutex::new(node));
    let mut vec = vec![node as Arc<Mutex<dyn Node<_> + Send + Sync>>];
    for i in 0..1000 {
        let node = Sine::new(Constant::from(220.0 + 10.0 * i as f64));
        let node = Arc::new(Mutex::new(Amp::new(node, Constant::from(0.001))));
        vec.push(node);
        // vec.push(node as Arc<Mutex<dyn Node<_> + Send + Sync>>);
    }
    let mix = ParallelMix::new(vec);
    let node = mix;
    write_to_file::write_to_file("parallel_mix.wav", 44100, 3.0, node, None, None);
}
