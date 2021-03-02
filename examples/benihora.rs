mod write_to_file;

use std::sync::Arc;

use corus::{
    contrib::{
        amp_pan,
        benihora::{make_noise_node, Benihora, BenihoraEvent},
    },
    core::constant::Constant,
    notenum_to_frequency, EventQueue,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut eq = EventQueue::new();
    let benihora = Arc::new(Benihora::new(make_noise_node(), 2));
    let mut benihora_ctl = eq.get_controller(&benihora);
    // let mut benihora = EventControl::new(benihora);
    benihora_ctl.push(0.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    benihora_ctl.push(1.0, BenihoraEvent::MoveTangue(19.4, 3.43));
    benihora_ctl.push(2.0, BenihoraEvent::MoveTangue(22.8, 2.05));
    benihora_ctl.push(3.0, BenihoraEvent::MoveTangue(27.2, 2.2));
    benihora_ctl.push(4.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora_ctl.push(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.6)]));
    benihora_ctl.push(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora_ctl.push(5.1, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora_ctl.push(5.2, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora_ctl.push(5.3, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora_ctl.push(5.4, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora_ctl.push(5.5, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora_ctl.push(5.6, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.3)]));
    benihora_ctl.push(5.7, BenihoraEvent::SetOtherConstrictions(vec![]));

    // benihora_ctl.push(5.0, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora_ctl.push(5.1, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora_ctl.push(5.2, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora_ctl.push(5.3, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora_ctl.push(5.4, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora_ctl.push(5.5, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora_ctl.push(5.6, BenihoraEvent::MoveTangue(36.0, 0.3));
    // benihora_ctl.push(5.7, BenihoraEvent::MoveTangue(12.9, 2.43));

    // benihora_ctl.push(0.0, BenihoraEvent::SetFrequency(440.0));
    benihora_ctl.push(
        6.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(48) as f64),
    );
    benihora_ctl.push(
        6.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(50) as f64),
    );
    benihora_ctl.push(
        7.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(52) as f64),
    );
    benihora_ctl.push(
        7.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(53) as f64),
    );
    benihora_ctl.push(
        8.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(55) as f64),
    );
    benihora_ctl.push(
        8.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(57) as f64),
    );
    benihora_ctl.push(
        9.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(59) as f64),
    );
    benihora_ctl.push(
        9.5,
        BenihoraEvent::SetFrequency(notenum_to_frequency(60) as f64),
    );
    benihora_ctl.push(
        10.0,
        BenihoraEvent::SetFrequency(notenum_to_frequency(48) as f64),
    );

    benihora_ctl.push(7.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    benihora_ctl.push(7.5, BenihoraEvent::MoveTangue(19.4, 3.43));
    benihora_ctl.push(8.0, BenihoraEvent::MoveTangue(22.8, 2.05));
    benihora_ctl.push(8.5, BenihoraEvent::MoveTangue(27.2, 2.2));
    benihora_ctl.push(9.0, BenihoraEvent::MoveTangue(12.9, 2.43));

    benihora_ctl.push(10.0, BenihoraEvent::SetTenseness(0.0));
    benihora_ctl.push(10.5, BenihoraEvent::SetTenseness(0.25));
    benihora_ctl.push(11.0, BenihoraEvent::SetTenseness(0.5));
    benihora_ctl.push(11.5, BenihoraEvent::SetTenseness(0.75));
    benihora_ctl.push(12.0, BenihoraEvent::SetTenseness(1.0));
    benihora_ctl.push(12.5, BenihoraEvent::SetTenseness(0.6));

    benihora_ctl.push(13.0, BenihoraEvent::SetStatus(false, false));
    benihora_ctl.push(14.0, BenihoraEvent::SetStatus(true, false));

    benihora_ctl.push(
        15.0,
        BenihoraEvent::SetOtherConstrictions(vec![(12.0, 0.6)]),
    );
    benihora_ctl.push(16.0, BenihoraEvent::SetOtherConstrictions(vec![]));

    let node = amp_pan(benihora, Constant::from(1.0), Constant::from(0.0));
    write_to_file::write_to_file("benihora.wav", SAMPLE_RATE, 20.0, eq.finish(node));
}
