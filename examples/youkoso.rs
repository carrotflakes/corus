mod write_to_file;

use corus::{
    contrib::{
        amp_pan,
        chip::{Noise, NoiseEvent},
        controllable_param, delay_fx,
        envelope::{AdsrEnvelope, ArEnvelope},
        event_controll::EventControll,
        poly_synth::{PolySynth, Voice},
        rand_fm_synth::rand_fm_synth,
        resetable_acc,
    },
    core::{
        add::Add, amp::Amp, constant::Constant, controllable::Controllable, mix::Mix, param::Param,
        Node,
    },
    notenum_to_frequency,
    signal::{C1f64, C2f64},
};

const SAMPLE_RATE: usize = 44100;

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
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.0.note_on(e.time, notenum);
                    track.3 = true;
                }
                ezmid::EventBody::NoteOff {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.0.note_off(e.time, notenum);
                }
                ezmid::EventBody::Volume { volume, .. } => {
                    track.1.set_value_at_time(e.time, volume as f64);
                }
                ezmid::EventBody::Pan { pan, .. } => {
                    track.2.set_value_at_time(e.time, pan as f64);
                }
                ezmid::EventBody::PitchBend {
                    bend: _,
                    raw_bend: _,
                } => {}
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
        .map(|t| Box::new(amp_pan(t.0, t.1, t.2)) as Box<dyn Node<C2f64>>)
        .collect();

    println!("{} synthes", synthes.len());

    let mix = Mix::new(synthes);
    let node = Amp::new(mix, Constant::new(C2f64([0.25, 0.25])));
    let node = delay_fx(node, SAMPLE_RATE as usize, 0.3, 0.3);

    let file = format!("{}.wav", file[..file.len() - 4].to_string());
    write_to_file::write_to_file(file.as_str(), SAMPLE_RATE, time, node);
    println!("saved {:?}", &file);
}

fn new_track(
    track: usize,
    program: u8,
) -> (
    Box<PolySynth<dyn Node<C1f64>, Box<dyn Node<C1f64>>>>,
    Param<f64, C1f64>,
    Param<f64, C1f64>,
    bool,
) {
    let synth = if track == 9 {
        Box::new(PolySynth::new(&noise_builder, 8))
            as Box<PolySynth<dyn Node<C1f64>, Box<dyn Node<C1f64>>>>
    } else {
        // Box::new(PolySynth::new(&|| fm_synth_builder(program as u32), 8))
        Box::new(PolySynth::new(&saw_builder, 8))
            as Box<PolySynth<dyn Node<C1f64>, Box<dyn Node<C1f64>>>>
    };
    let gain = Param::with_value(1.0);
    let pan = Param::with_value(0.0);
    (synth, gain, pan, false)
}

fn saw_builder() -> Voice<dyn Node<C1f64>, Box<dyn Node<C1f64>>> {
    let (freq_param, mut freq_param_ctrl) = controllable_param(1.0);
    let (acc, mut acc_reset) = resetable_acc(freq_param);
    let saw = Add::new(acc, Constant::from(-0.5));
    let (env, mut env_on, env_off) = AdsrEnvelope::new(
        0.01,
        0.5,
        0.2,
        0.3,
    )
    .build();
    let node = Amp::new(saw, env);
    Voice::new(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, notenum| {
            freq_param_ctrl
                .lock()
                .set_value_at_time(time, notenum_to_frequency(notenum as u32));
            acc_reset(time, 0.5);
            env_on(time);
        }),
        Box::new(env_off),
    )
}

fn noise_builder() -> Voice<dyn Node<C1f64>, Box<dyn Node<C1f64>>> {
    let noise = Controllable::new(EventControll::new(Noise::new()));
    let mut noise_ctrl = noise.controller();
    let (env, mut env_on, env_off) = ArEnvelope::new(0.01, 0.3).build();
    let node = Amp::new(noise, Amp::new(env, Constant::from(0.25)));
    Voice::new(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, notenum| {
            noise_ctrl.lock().push_event(time, NoiseEvent::ResetReg);
            noise_ctrl.lock().push_event(
                time,
                NoiseEvent::OriginalFreq(notenum % 7, (15 * notenum as usize / 127) as u8),
            );
            env_on(time)
        }),
        Box::new(env_off),
    )
}

fn fm_synth_builder(seed: u32) -> Voice<dyn Node<C1f64>, Box<dyn Node<C1f64>>> {
    let synth = Controllable::new(rand_fm_synth(seed));
    let mut ctrl1 = synth.controller();
    let mut ctrl2 = synth.controller();
    Voice::new(
        Box::new(synth) as Box<dyn Node<C1f64>>,
        Box::new(move |time, notenum| {
            ctrl1
                .lock()
                .note_on(time, notenum_to_frequency(notenum as u32))
        }),
        Box::new(move |time| {
            ctrl2.lock().note_off(time);
        }),
    )
}
