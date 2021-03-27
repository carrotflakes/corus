mod write_to_file;

use std::sync::{Arc, Mutex};

use corus::{
    contrib::{
        amp_pan,
        envelope2::AdsrEnvelope,
        generic_poly_synth::{NoteOff, NoteOn, PolySynth, Voice},
    },
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        add::Add,
        amp::Amp,
        var::Var,
        mix::Mix,
        mul::Mul,
        param3::{ParamEventSchedule, ParamEventScheduleNode},
        share::Share,
        Node,
    },
    db_to_amp, notenum_to_frequency,
    signal::C2f64,
    EventControllable, EventPusher, EventScheduleNode,
};

const SAMPLE_RATE: usize = 44100;
const DB_MIN: f64 = 24.0;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("./youkoso.mid".to_string());

    let mut tracks: Vec<_> = (0..16).map(|i| Track::new(i, 0)).collect();

    let time = {
        let data = std::fs::read(&file).unwrap();
        let events = ezmid::parse(&data);
        let mut time = 0.0;
        for e in ezmid::Dispatcher::new(events) {
            time = e.time;

            let track = &mut tracks[e.event.channel as usize];
            match e.event.body {
                ezmid::EventBody::NoteOn {
                    notenum,
                    velocity,
                    raw_velocity: _,
                } => {
                    track
                        .synth
                        .note_on(e.time, Some(notenum), (notenum, velocity as f64));
                    track.used = true;
                }
                ezmid::EventBody::NoteOff {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.synth.note_off(e.time, Some(notenum), ());
                }
                ezmid::EventBody::Volume { volume, .. } => {
                    track
                        .gain
                        .get_scheduler()
                        .lock()
                        .unwrap()
                        .set_value_at_time(e.time, db_to_amp((volume as f64 - 1.0) * DB_MIN));
                }
                ezmid::EventBody::Pan { pan, .. } => {
                    track
                        .pan
                        .get_scheduler()
                        .lock()
                        .unwrap()
                        .set_value_at_time(e.time, pan as f64);
                }
                ezmid::EventBody::PitchBend { bend, raw_bend: _ } => {
                    track
                        .pitch_ctl
                        .lock()
                        .unwrap()
                        .set_value_at_time(e.time, 2.0f64.powf(bend as f64 / 12.0));
                }
                ezmid::EventBody::Tempo { tempo: _ } => {}
                ezmid::EventBody::ProgramChange { program } => {
                    track.change_program(program);
                }
                _ => {}
            }
        }
        time + 1.0
    };

    let synthes: Vec<_> = tracks.into_iter().flat_map(|t| t.finish()).collect();

    println!("{} tracks", synthes.len());

    let node = Mix::new(synthes);

    let file = format!("{}.wav", file[..file.len() - 4].to_string());
    write_to_file::write_to_file(
        file.as_str(),
        SAMPLE_RATE,
        time,
        node,
        Some(0x51dd88ea14d905c1),
        Some(0xbc1eff14d2d27763),
    );
    println!("saved {:?}", &file);
}

type MyVoice = Voice<Box<dyn Node<Output = f64> + Send + Sync>, (u8, f64), ()>;

pub struct Track {
    track: usize,
    synth: PolySynth<(u8, f64), (), MyVoice, Option<u8>>,
    gain: ParamEventScheduleNode<f64>,
    pan: ParamEventScheduleNode<f64>,
    used: bool,
    pitch: Share<ParamEventScheduleNode<f64>>,
    pitch_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    synths: Vec<PolySynth<(u8, f64), (), MyVoice, Option<u8>>>,
}

impl Track {
    pub fn new(track: usize, program: u8) -> Self {
        let (pitch, pitch_ctl) = controllable_param(1.0);
        let pitch = Share::new(pitch);
        let gain = ParamEventScheduleNode::from_value(1.0f64);
        let pan = ParamEventScheduleNode::from_value(0.0f64);
        Self {
            track,
            synth: Self::make_synth(track, program, pitch.clone()),
            gain,
            pan,
            used: false,
            pitch,
            pitch_ctl,
            synths: vec![],
        }
    }

    fn make_synth(
        _track: usize,
        _program: u8,
        pitch: Share<ParamEventScheduleNode<f64>>,
    ) -> PolySynth<(u8, f64), (), MyVoice, Option<u8>> {
        PolySynth::new(&mut || saw_builder(pitch.clone()), 8)
    }

    pub fn change_program(&mut self, program: u8) {
        let mut synth = Self::make_synth(self.track, program, self.pitch.clone());
        std::mem::swap(&mut synth, &mut self.synth);
        if self.used {
            self.synths.push(synth);
            self.used = false;
        }
    }

    pub fn finish(mut self) -> Option<Box<dyn Node<Output = C2f64> + Send + Sync>> {
        if self.used {
            Some(if self.synths.is_empty() {
                Box::new(amp_pan(self.synth, self.gain, self.pan))
            } else {
                let mut nodes = self.synths;
                nodes.push(self.synth);
                Box::new(amp_pan(Mix::new(nodes), self.gain, self.pan))
            })
        } else {
            match self.synths.len() {
                0 => None,
                1 => Some(Box::new(amp_pan(
                    self.synths.pop().unwrap(),
                    self.gain,
                    self.pan,
                ))),
                _ => Some(Box::new(amp_pan(
                    Mix::new(self.synths),
                    self.gain,
                    self.pan,
                ))),
            }
        }
    }
}

fn saw_builder(pitch: Share<ParamEventScheduleNode<f64>>) -> MyVoice {
    let (freq, freq_ctl) = controllable_param(1.0);
    let (gain, gain_ctl) = controllable_param(1.0);

    let acc = EventScheduleNode::new(EventControllable::new(Accumulator::new(
        Mul::new(freq, pitch),
        1.0,
    )));
    let mut acc_ctl = acc.get_scheduler();
    let mut acc_reset = move |time: f64| acc_ctl.push_event(time, SetValueAtTime::new(0.5));

    let saw = Add::new(acc, Var::from(-0.5));
    let (env, mut env_on, mut env_off) = AdsrEnvelope::<f64>::new(0.01, 0.5, 0.2, 0.3).build();
    let node = Amp::new(saw, Amp::new(env, gain));
    Voice(
        Box::new(node) as Box<dyn Node<Output = f64> + Send + Sync>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            freq_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, notenum_to_frequency(notenum));
            gain_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, db_to_amp((velocity - 1.0) * DB_MIN));
            acc_reset(time);
            env_on(time);
        }),
        Box::new(move |time, NoteOff(())| env_off(time)),
    )
}

pub fn controllable_param(
    v: f64,
) -> (
    ParamEventScheduleNode<f64>,
    Arc<Mutex<ParamEventSchedule<f64>>>,
) {
    let mut c = ParamEventScheduleNode::from_value(v);
    let ctl = c.get_scheduler();
    (c, ctl)
}
