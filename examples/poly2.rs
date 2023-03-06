mod write_to_file;

use corus::{
    contrib::{
        envelope2::AdsrEnvelope,
        generic_poly_synth::{Event, PolySynth, Voice},
    },
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        add::Add,
        amp::Amp,
        param3::ParamEventScheduleNode,
        var::Var,
    },
    notenum_to_frequency,
    signal::C1f64,
    EventControlInplace, EventControllable, EventPusher, EventScheduleNode,
};

fn main() {
    let sample_rate = 44100;

    let mut builder = || {
        let freq = ParamEventScheduleNode::new();
        let freq_ctl = freq.get_scheduler();
        // freq_ctl.lock().unwrap().set_value_at_time(0.0, 440.0);
        let acc = EventScheduleNode::new(EventControllable::new(Accumulator::new(
            freq,
            C1f64::from(1.0),
        )));
        let mut acc_ctl = acc.get_scheduler();
        // let acc = Accumulator::new(freq, C1f64::from(1.0));
        let saw = Add::new(acc, Var::from(-0.5));
        let (env, mut env_on, mut env_off) = AdsrEnvelope::<f64>::new(0.01, 0.5, 0.2, 0.3).build();
        let node = Amp::new(saw, env);
        // let node = saw;
        Voice(
            node,
            Box::new(move |time, notenum| {
                freq_ctl
                    .lock()
                    .unwrap()
                    .set_value_at_time(time, notenum_to_frequency(notenum));
                acc_ctl.push_event(time, SetValueAtTime::new(C1f64::from(0.5)));
                env_on(time);
            }),
            Box::new(move |time, ()| {
                env_off(time);
            }),
        )
    };
    let synth = PolySynth::new(&mut builder, 10);
    let synth = EventScheduleNode::new(EventControllable::new(synth));

    let mut scheduler = synth.get_scheduler();
    scheduler.push_event(0.2, Event::NoteOn((60, 60)));
    scheduler.push_event(0.4, Event::NoteOff((60, ())));
    scheduler.push_event(0.4, Event::NoteOn((64, 64)));
    scheduler.push_event(0.6, Event::NoteOff((64, ())));
    scheduler.push_event(0.6, Event::NoteOn((67, 67)));
    scheduler.push_event(0.8, Event::NoteOff((67, ())));
    scheduler.push_event(0.8, Event::NoteOn((64, 64)));
    scheduler.push_event(1.0, Event::NoteOff((64, ())));
    scheduler.push_event(1.0, Event::NoteOn((60, 60)));
    scheduler.push_event(1.6, Event::NoteOff((60, ())));
    scheduler.push_event(1.1, Event::NoteOn((64, 64)));
    scheduler.push_event(1.6, Event::NoteOff((64, ())));
    scheduler.push_event(1.2, Event::NoteOn((67, 67)));
    scheduler.push_event(1.6, Event::NoteOff((67, ())));
    scheduler.push_event(1.3, Event::NoteOn((71, 71)));
    scheduler.push_event(1.6, Event::NoteOff((71, ())));

    let node = Amp::new(synth, Var::from(0.1));

    write_to_file::write_to_file("poly.wav", sample_rate, 3.0, node, None, None);
}
