mod write_to_file;

use corus::{
    contrib::{
        chip::{Noise, NoiseEvent},
        event_control::EventControl,
    },
    signal::C1f64,
};

fn main() {
    let sample_rate = 44100;

    let mut node: EventControl<C1f64, NoiseEvent> = EventControl::new(Noise::new());
    node.push_event(2.0 * 0.0, NoiseEvent::ShortFreq(false));
    node.push_event(2.0 * 0.1, NoiseEvent::OriginalFreq(1, 4));
    node.push_event(2.0 * 0.2, NoiseEvent::OriginalFreq(2, 4));
    node.push_event(2.0 * 0.3, NoiseEvent::OriginalFreq(3, 4));
    node.push_event(2.0 * 0.4, NoiseEvent::OriginalFreq(4, 4));
    node.push_event(2.0 * 0.5, NoiseEvent::OriginalFreq(5, 4));
    node.push_event(2.0 * 0.6, NoiseEvent::OriginalFreq(6, 4));
    node.push_event(2.0 * 0.7, NoiseEvent::OriginalFreq(7, 4));
    write_to_file::write_to_file("chip_noise.wav", sample_rate, 3.0, node);
}
