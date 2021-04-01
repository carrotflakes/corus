//! Usage: cargo run --release --example star | pacat

mod write_to_file;

use std::sync::{Arc, Mutex};

use corus::{
    contrib::envelope3::{Envelope, EnvelopeEvent},
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        amp::Amp,
        map::Map,
        mix::Mix,
        mul::Mul,
        param3::{ParamEventSchedule, ParamEventScheduleNode},
        var::Var,
    },
    db_to_amp, notenum_to_frequency,
    signal::{IntoStereo, Stereo},
    time::Second,
    EventControllable, EventPusher, EventSchedule, EventScheduleNode, Node, ProcContext,
};

const DB_MIN: f64 = 24.0;

fn main() {
    let (node1, mut noteon1) = square();
    let (node2, mut noteon2) = square();
    let (node3, mut noteon3) = triangle();
    let node = Mix::new(vec![node1, node2, node3]);
    let node = Amp::new(node, Var::new(0.1));

    let mut i = 0;
    render_loop(node, &mut |next_time| loop {
        let time = i as f64 * 1.6 * 2.0;
        if next_time < time {
            break;
        }
        for (t, d) in &[(0, 2), (2, 2), (4, 3), (7, 2), (9, 2), (12, 2), (14, 3)] {
            let t = time + *t as f64 * 0.1;
            let d = *d as f64 * 0.05 - 0.001;
            noteon1(t, d, 72, 0.5);
            noteon2(t, d, 65, 0.5);
            noteon1(t + 1.6, d, 72 - 1, 0.5);
            noteon2(t + 1.6, d, 65 - 1, 0.5);
        }
        noteon2(time + 1.10, 0.1, 65, 0.5);
        noteon2(time + 1.60 + 1.10, 0.1, 65 - 1, 0.5);
        for (t, d, n) in &[
            (0, 3, 50),
            (4, 3, 57),
            (6, 2, 62),
            (7, 2, 62),
            (11, 2, 62),
            (12, 2, 57),
            (13, 2, 62),
            (14, 3, 62),
        ] {
            let t = time + *t as f64 * 0.1;
            let d = *d as f64 * 0.05 - 0.001;
            noteon3(t, d, *n, 0.6);
            noteon3(t + 1.6, d, *n - 2, 0.6);
        }
        i += 1;
    });
}

fn render_loop(mut node: impl Node<Output = f64>, schedule: &mut impl FnMut(f64)) {
    let mut pc = ProcContext::new(44100 as u64);

    use std::io::{self, BufWriter, Write};
    let stdout = io::stdout();
    let mut handle = BufWriter::new(stdout.lock());
    loop {
        schedule(pc.current_time + 0.1);
        for s in pc.lock(&mut node, Second(0.1)) {
            let s = s.into_stereo();
            if !(s.get_l() as f64).is_finite() || !(s.get_r() as f64).is_finite() {
                panic!(
                    "signal is not finite, l: {:?}, r: {:?}",
                    s.get_l(),
                    s.get_r()
                );
            }
            let l = (s.get_l() * std::i16::MAX as f64) as i16;
            let r = (s.get_r() * std::i16::MAX as f64) as i16;
            handle.write_all(&l.to_ne_bytes()).unwrap();
            handle.write_all(&r.to_ne_bytes()).unwrap();
            handle.flush().unwrap();
        }
    }
}

fn square() -> (
    Box<dyn Node<Output = f64> + Send + Sync>,
    Box<dyn FnMut(f64, f64, u8, f64)>,
) {
    let (freq, freq_ctl) = controllable_param(1.0);
    let (gain, gain_ctl) = controllable_param(1.0);

    let (acc, mut acc_ctl) = controllable(Accumulator::new(freq, 1.0));

    let node = Map::new(acc, |x| if x < 0.5 { -1.0 } else { 1.0 });
    let (env, mut env_ctl1) = controllable(Envelope::new(
        &[(0.01, 1.0, -1.0), (0.2, 0.9, 1.0)],
        0.01,
        1.0,
    ));
    let mut env_ctl2 = env_ctl1.clone();
    let node = Amp::new(node, Mul::new(env, gain));
    (
        Box::new(node) as Box<dyn Node<Output = f64> + Send + Sync>,
        Box::new(move |time, dur, notenum, velocity| {
            freq_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, notenum_to_frequency(notenum));
            gain_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, db_to_amp((velocity - 1.0) * DB_MIN));
            acc_ctl.push_event(time, SetValueAtTime::new(0.5));
            env_ctl1.push_event(time, EnvelopeEvent::NoteOn);
            env_ctl2.push_event(time + dur, EnvelopeEvent::NoteOff);
        }),
    )
}

fn triangle() -> (
    Box<dyn Node<Output = f64> + Send + Sync>,
    Box<dyn FnMut(f64, f64, u8, f64)>,
) {
    let (freq, freq_ctl) = controllable_param(1.0);
    let (gain, gain_ctl) = controllable_param(1.0);

    let (acc, mut acc_ctl) = controllable(Accumulator::new(freq, 1.0));

    let node = Map::new(acc, |x| {
        (if x < 0.5 {
            x * 4.0 - 1.0
        } else {
            3.0 - x * 4.0
        } * 9.0) as i8 as f64
    });
    let (env, mut env_ctl1) = controllable(Envelope::new(
        &[(0.01, 1.0, -1.0), (0.2, 0.9, 1.0)],
        0.01,
        1.0,
    ));
    let mut env_ctl2 = env_ctl1.clone();
    let node = Amp::new(node, Mul::new(env, gain));
    (
        Box::new(node) as Box<dyn Node<Output = f64> + Send + Sync>,
        Box::new(move |time, dur, notenum, velocity| {
            freq_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, notenum_to_frequency(notenum));
            gain_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, db_to_amp((velocity - 1.0) * DB_MIN));
            acc_ctl.push_event(time, SetValueAtTime::new(0.5));
            env_ctl1.push_event(time, EnvelopeEvent::NoteOn);
            env_ctl2.push_event(time + dur, EnvelopeEvent::NoteOff);
        }),
    )
}

pub fn controllable_param(
    v: f64,
) -> (
    ParamEventScheduleNode<f64>,
    Arc<Mutex<ParamEventSchedule<f64>>>,
) {
    let param = ParamEventScheduleNode::from_value(v);
    let ctl = param.get_scheduler();
    (param, ctl)
}

pub fn controllable<A: Node>(node: A) -> (EventScheduleNode<A>, EventSchedule<A>) {
    let node = EventScheduleNode::new(EventControllable::new(node));
    let ctl = node.get_scheduler();
    (node, ctl)
}
