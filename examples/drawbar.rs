mod write_to_file;

use corus::{
    core::{
        mix::Mix,
        mul::Mul,
        share::Share,
        sine::Sine,
        var::{Var, VarEvent},
    },
    notenum_to_frequency, EventControlInplace, EventPusher,
};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let mut freq = EventControlInplace::new(Var::from(notenum_to_frequency(60)));
    freq.push_event(0.25, VarEvent::SetValue(notenum_to_frequency(64)));
    freq.push_event(0.5, VarEvent::SetValue(notenum_to_frequency(67)));
    freq.push_event(0.75, VarEvent::SetValue(notenum_to_frequency(64)));
    freq.push_event(1.0, VarEvent::SetValue(notenum_to_frequency(70)));
    freq.push_event(1.25, VarEvent::SetValue(notenum_to_frequency(69)));
    freq.push_event(1.3, VarEvent::SetValue(notenum_to_frequency(70)));
    freq.push_event(1.4, VarEvent::SetValue(notenum_to_frequency(69)));
    freq.push_event(1.5, VarEvent::SetValue(notenum_to_frequency(67)));
    let freq = Share::new(freq);

    let sine1 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(1.0))),
        Var::from(0.2),
    );
    let sine2 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(2.0))),
        Var::from(0.2),
    );
    let sine3 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(3.0))),
        Var::from(0.2),
    );
    let sine4 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(4.0))),
        Var::from(0.1),
    );
    let sine5 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(5.0))),
        Var::from(0.05),
    );
    let sine6 = Mul::new(
        Sine::new(Mul::new(freq.clone(), Var::from(6.0))),
        Var::from(0.025),
    );

    let node = Mix::new(vec![sine1, sine2, sine3, sine4, sine5, sine6]);
    write_to_file::write_to_file("drawbar.wav", SAMPLE_RATE, 2.0, node, None, None);
}
