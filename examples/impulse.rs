mod write_to_file;

use corus::{
    contrib::{bypass_fader::bypass_fader, schroeder::schroeder_reverb},
    core::{
        impulse::{Impulse, ImpulseEvent},
        param::Param,
        proc_once_share::ProcOnceShare,
    },
    EventControlInplace, EventPusher,
};

fn main() {
    let mut impulse = EventControlInplace::new(Impulse::new(0.5, false));
    impulse.push_event(0.5, ImpulseEvent::Trigger);
    impulse.push_event(1.0, ImpulseEvent::Trigger);
    impulse.push_event(1.25, ImpulseEvent::Trigger);
    impulse.push_event(1.5, ImpulseEvent::Trigger);
    impulse.push_event(2.0, ImpulseEvent::Trigger);
    impulse.push_event(2.5, ImpulseEvent::Trigger);
    impulse.push_event(3.0, ImpulseEvent::Trigger);
    impulse.push_event(3.25, ImpulseEvent::Trigger);
    impulse.push_event(3.5, ImpulseEvent::Trigger);
    let mut fade = Param::with_value(0.0);
    fade.set_value_at_time(2.0, 1.0);
    let node = bypass_fader(
        ProcOnceShare::new(impulse),
        &|node| schroeder_reverb(node),
        fade,
    );
    write_to_file::write_to_file("impulse.wav", 44100, 5.0, node);
}
