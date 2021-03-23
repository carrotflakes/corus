mod write_to_file;

use corus::{
    contrib::{amp_pan, chip::Noise},
    core::{
        add::Add,
        amp::Amp,
        biquad_filter::{BiquadFilter, BiquadFilterParams},
        constant::Constant,
        param::Param,
        placeholder::Placeholder,
        share::Share,
        ring_buffer_playback::RingBufferPlayback,
        ring_buffer_record::RingBufferRecord,
        Node,
    },
    signal::C1f64,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let node = Noise::new();
    let mut env = Param::new();
    env.set_value_at_time(0.0, 1.0);
    env.set_value_at_time(0.01, 0.0);
    let node = Amp::new(node, env);

    let node = {
        let mut p = Placeholder::new(None);
        let mut ps = p.setter();
        let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
        unsafe {
            ps.set(Box::new(Add::new(
                node,
                Amp::new(
                    BiquadFilter::new(
                        RingBufferPlayback::new(Constant::from(0.01), buffer.clone()),
                        BiquadFilterParams::new(
                            corus::core::biquad_filter::LowPass,
                            Constant::from(880.0),
                            Constant::from(1.0),
                            Constant::from(2.0),
                        ),
                    ),
                    Constant::from(0.5),
                ),
            )) as Box<dyn Node<Output = C1f64>>);
        }
        buffer
    };
    let node = amp_pan(node, Constant::from(0.5), Constant::from(0.0));

    write_to_file::write_to_file("karplus-strong.wav", SAMPLE_RATE, 3.0, node, None, None);
}
