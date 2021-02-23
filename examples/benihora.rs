mod write_to_file;

use corus::{
    contrib::{
        amp_pan,
        benihora::{make_noise_node, Benihora, BenihoraEvent},
        event_controll::EventControll,
    },
    node::map::Map,
    signal::C1f32,
};

use corus::node::{self};
use node::constant::Constant;

const SAMPLE_RATE: usize = 44100;

fn main() {
    let benihora = Benihora::new(make_noise_node());
    let mut benihora = EventControll::new(benihora);
    benihora.push_event(0.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    benihora.push_event(1.0, BenihoraEvent::MoveTangue(19.4, 3.43));
    benihora.push_event(2.0, BenihoraEvent::MoveTangue(22.8, 2.05));
    benihora.push_event(3.0, BenihoraEvent::MoveTangue(27.2, 2.2));
    benihora.push_event(4.0, BenihoraEvent::MoveTangue(12.9, 2.43));
    // benihora.push_event(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.6)]));
    benihora.push_event(5.0, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.0)]));
    benihora.push_event(5.1, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora.push_event(5.2, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.0)]));
    benihora.push_event(5.3, BenihoraEvent::SetOtherConstrictions(vec![]));
    benihora.push_event(5.4, BenihoraEvent::SetOtherConstrictions(vec![(36.0, 0.0)]));
    benihora.push_event(5.5, BenihoraEvent::SetOtherConstrictions(vec![]));
    let node = amp_pan(
        Map::new(benihora, |c| C1f32([c.0[0] as f32])),
        Constant::from(1.0),
        Constant::from(0.0),
    );
    write_to_file::write_to_file("benihora.wav", SAMPLE_RATE, 10.0, node);
}
