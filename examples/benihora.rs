mod write_to_file;

use corus::{
    contrib::{
        amp_pan,
        benihora::{make_noise_node, Benihora, BenihoraEvent},
        event_controll::EventControll,
    },
    node::map::Map,
    notenum_to_frequency,
    signal::C1f32,
};

use corus::node::{self};
use node::constant::Constant;

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut benihora = Benihora::new(make_noise_node());
    benihora.twice = true;
    let mut benihora = EventControll::new(benihora);
    benihora.push_event(0.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    benihora.push_event(1.0, BenihoraEvent::MoveTangue(19.4, 3.43));
    benihora.push_event(2.0, BenihoraEvent::MoveTangue(22.8, 2.05));
    benihora.push_event(3.0, BenihoraEvent::MoveTangue(27.2, 2.2));
    benihora.push_event(4.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora.push_event(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.6)]));
    benihora.push_event(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora.push_event(5.1, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora.push_event(5.2, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora.push_event(5.3, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora.push_event(5.4, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora.push_event(5.5, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora.push_event(5.6, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora.push_event(5.7, BenihoraEvent::SetOtherConstrictions(vec![]));

    // benihora.push_event(5.0, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora.push_event(5.1, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora.push_event(5.2, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora.push_event(5.3, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora.push_event(5.4, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora.push_event(5.5, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora.push_event(5.6, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora.push_event(5.7, BenihoraEvent::MoveTangue(12.9, 2.43));

    // benihora.push_event(0.0, BenihoraEvent::SetFrequency(440.0));
    benihora.push_event(
        6.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(48) as f64),
    );
    benihora.push_event(
        6.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(50) as f64),
    );
    benihora.push_event(
        7.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(52) as f64),
    );
    benihora.push_event(
        7.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(53) as f64),
    );
    benihora.push_event(
        8.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(55) as f64),
    );
    benihora.push_event(
        8.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(57) as f64),
    );
    benihora.push_event(
        9.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(59) as f64),
    );
    benihora.push_event(
        9.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(60) as f64),
    );
    benihora.push_event(
        10.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(48) as f64),
    );

    benihora.push_event(7.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    benihora.push_event(7.5, BenihoraEvent::MoveTangue(19.4, 3.43));
    benihora.push_event(8.0, BenihoraEvent::MoveTangue(22.8, 2.05));
    benihora.push_event(8.5, BenihoraEvent::MoveTangue(27.2, 2.2));
    benihora.push_event(9.0, BenihoraEvent::MoveTangue(12.9, 2.43));

    benihora.push_event(10.0, BenihoraEvent::SetTenseness(0.0));
    benihora.push_event(10.5, BenihoraEvent::SetTenseness(0.25));
    benihora.push_event(11.0, BenihoraEvent::SetTenseness(0.5));
    benihora.push_event(11.5, BenihoraEvent::SetTenseness(0.75));
    benihora.push_event(12.0, BenihoraEvent::SetTenseness(1.0));
    benihora.push_event(12.5, BenihoraEvent::SetTenseness(0.6));

    benihora.push_event(13.0, BenihoraEvent::SetStatus(false, false));
    benihora.push_event(14.0, BenihoraEvent::SetStatus(true, false));

    benihora.push_event(15.0, BenihoraEvent::SetOtherConstrictions(vec![(12.0, 0.6)]));
    benihora.push_event(16.0, BenihoraEvent::SetOtherConstrictions(vec![]));

    let node = amp_pan(
        Map::new(benihora, |c| C1f32([c.0[0] as f32])),
        Constant::from(1.0),
        Constant::from(0.0),
    );
    write_to_file::write_to_file("benihora.wav", SAMPLE_RATE, 20.0, node);
}
