mod write_to_file;

use corus::{EventControllable, EventPusher, EventQueue, contrib::{
        amp_pan,
        benihora::{make_noise_node, Benihora, BenihoraEvent},
    }, core::{var::Var, controllable::Controllable}, notenum_to_frequency};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut eq = EventQueue::new();
    let benihora = EventControllable::new(Benihora::new(make_noise_node(), 2));
    let mut benihora_ctl = eq.get_controller(&benihora);
    benihora_ctl.push_event(1.0, BenihoraEvent::SetStatus(false, false));

    let node = amp_pan(benihora, Var::from(1.0), Var::from(0.0));
    write_to_file::write_to_file_with_event_queue("benihora2.wav", SAMPLE_RATE, 20.0, node, None, None, eq);
}
