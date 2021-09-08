mod write_to_file;

use std::sync::{Arc, Mutex};

use corus::{
    contrib::{
        amp_pan,delay_fx,
        envelope::AdsrEnvelope,
        generic_poly_synth::{NoteOff, NoteOn, PolySynth, Voice},
        resetable_acc,
    },
    core::{
        add::Add,
        amp::Amp,
        var::Var,
        mix::Mix,
        param::Param,
        param3::{ParamEventSchedule, ParamEventScheduleNode},
        share::Share,
        Node,
    },
    db_to_amp, notenum_to_frequency,
    signal::C2f64,
};

const SAMPLE_RATE: usize = 44100;
const DB_MIN: f64 = 24.0;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("./youkoso.mid".to_string());

    let mut tracks: Vec<_> = (0..16).map(|i| new_track(i, 0)).collect();
    let mut used_tracks = Vec::new();

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
                        .0
                        .note_on(e.time, Some(notenum), (notenum, velocity as f64));
                    track.3 = true;
                }
                ezmid::EventBody::NoteOff {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.0.note_off(e.time, Some(notenum), ());
                }
                ezmid::EventBody::Volume { volume, .. } => {
                    track
                        .1
                        .set_value_at_time(e.time, db_to_amp((volume as f64 - 1.0) * DB_MIN));
                }
                ezmid::EventBody::Pan { pan, .. } => {
                    track.2.set_value_at_time(e.time, pan as f64);
                }
                ezmid::EventBody::PitchBend {
                    // TODO remains pitchbend after program changes
                    bend,
                    raw_bend: _,
                } => {
                    track
                        .4
                        .lock()
                        .unwrap()
                        .set_value_at_time(e.time, 2.0f64.powf(bend as f64 / 12.0));
                }
                ezmid::EventBody::Tempo { tempo: _ } => {}
                ezmid::EventBody::ProgramChange { program } => {
                    let mut track_ = new_track(e.event.channel as usize, program);
                    std::mem::swap(track, &mut track_);
                    used_tracks.push(track_);
                }
                _ => {}
            }
        }
        time + 1.0
    };

    let synthes: Vec<_> = tracks
        .into_iter()
        .chain(used_tracks)
        .filter(|t| t.3)
        .map(|t| Box::new(amp_pan(t.0, t.1, t.2)) as Box<dyn Node<Output = C2f64> + Send + Sync>)
        .collect();

    println!("{} synthes", synthes.len());

    let node = Mix::new(synthes);
    // let node = Amp::new(node, Var::new(C2f64([0.25, 0.25])));
    let node = delay_fx(node, SAMPLE_RATE as usize, 0.3, 0.3);

    let file = format!("{}.wav", file[..file.len() - 4].to_string());
    write_to_file::write_to_file(
        file.as_str(),
        SAMPLE_RATE,
        time,
        node,
        Some(0xe18828b8b469c40f),
        Some(0x704d5860b5877c58),
    );
    println!("saved {:?}", &file);
}

type MyVoice = Voice<Box<dyn Node<Output = f64> + Send + Sync>, (u8, f64), ()>;

fn new_track(
    _track: usize,
    _program: u8,
) -> (
    PolySynth<(u8, f64), (), MyVoice, Option<u8>>,
    Param<f64>,
    Param<f64>,
    bool,
    Arc<Mutex<ParamEventSchedule<f64>>>,
    // Controller<Param<f64>>,
) {
    // let (pitch, pitch_ctrl) = controllable_param(1.0);
    let mut pitch = ParamEventScheduleNode::new();
    let pitch_ctrl = pitch.get_scheduler();
    let pitch = Share::new(pitch);
    let synth = PolySynth::new(&mut || saw_builder(pitch.clone()), 8);
    let gain = Param::with_value(1.0f64);
    let pan = Param::with_value(0.0f64);
    (synth, gain, pan, false, pitch_ctrl)
}

fn saw_builder(pitch: Share<ParamEventScheduleNode<f64>>) -> MyVoice {
    // let (freq_param, mut freq_param_ctrl) = controllable_param(1.0);
    // let (gain, mut gain_ctrl) = controllable_param(1.0);
    let mut freq_param = ParamEventScheduleNode::new();
    let freq_param_ctrl = freq_param.get_scheduler();
    let mut gain = ParamEventScheduleNode::new();
    let gain_ctrl = gain.get_scheduler();
    let (acc, mut acc_reset) = resetable_acc(Amp::new(freq_param, pitch));
    let saw = Add::new(acc, Var::from(-0.5));
    let (env, mut env_on, mut env_off) = AdsrEnvelope::new(0.01, 0.5, 0.2, 0.3).build();
    let node = Amp::new(saw, Amp::new(env, gain));
    Voice(
        Box::new(node) as Box<dyn Node<Output = f64> + Send + Sync>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            freq_param_ctrl
                .lock()
                .unwrap()
                .set_value_at_time(time, notenum_to_frequency(notenum));
            gain_ctrl
                .lock()
                .unwrap()
                .set_value_at_time(time, db_to_amp((velocity - 1.0) * DB_MIN));
            acc_reset(time, 0.5);
            env_on(time);
        }),
        Box::new(move |time, NoteOff(())| env_off(time)),
    )
}
