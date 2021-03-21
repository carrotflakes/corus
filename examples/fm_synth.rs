mod write_to_file;

use corus::{
    contrib::{envelope2::AdsrEnvelope, fm_synth::FmSynth},
    core::constant::Constant,
};

fn main() {
    let sample_rate = 44100;

    #[rustfmt::skip]
    let mut node = FmSynth::new([
        (Constant::new(1.0f64), Constant::from(0.0), AdsrEnvelope::new(0.01, 0.5, 0.3, 0.3), [0.0; 4]),
        (Constant::from(1.01), Constant::from(0.0), AdsrEnvelope::new(0.5, 0.5, 0.7, 0.3), [0.0; 4]),
        (Constant::from(0.0), Constant::from(4.0), AdsrEnvelope::new(0.1, 0.5, 0.3, 0.3), [0.0; 4]),
        (Constant::from(1.0), Constant::from(0.0), AdsrEnvelope::new(0.02, 0.5, 0.3, 0.3), [0.0, 2000.0, 5.0, 0.0]),
    ],
    [0.0, 0.0, 0.0, 1.0],
    );
    node.note_on(0.0, 440.0);
    node.note_off(2.0);
    write_to_file::write_to_file("fm_synth.wav", sample_rate, 3.0, node, None, None);
}
