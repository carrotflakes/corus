mod write_to_file;

use corus::{
    contrib::{
        amp_pan,
        chip::{Noise, NoiseEvent},
        controllable_param, delay_fx,
        envelope::{AdsrEnvelope, ArEnvelope},
        generic_poly_synth::{NoteOff, NoteOn, PolySynth, Voice},
        perlin_noise,
        rand_fm_synth::rand_fm_synth,
        resetable_acc,
    },
    core::{
        accumulator::Accumulator,
        add::Add,
        amp::Amp,
        biquad_filter::{BiquadFilter, BiquadFilterParams, LowPass},
        constant::Constant,
        controllable::{Controllable, Controller},
        map::Map,
        mix::Mix,
        param::Param,
        proc_once_share::ProcOnceShare,
        Node,
    },
    notenum_to_frequency,
    signal::{C1f64, C2f64},
    EventControlInplace, ProcContext,
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
                    track.1.set_value_at_time(e.time, volume as f64);
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

type MyVoice = Voice<Box<dyn Node<f64>>, (u8, f64), ()>;

fn new_track(
    track: usize,
    program: u8,
) -> (
    PolySynth<(u8, f64), (), MyVoice, Option<u8>>,
    Param<f64, f64>,
    Param<f64, f64>,
    bool,
    Controller<f64, Param<f64, f64>>,
) {
    let (pitch, pitch_ctrl) = controllable_param(1.0);
    let pitch = ProcOnceShare::new(pitch);
    let synth = if track == 0 || track == 2 || track == 3 {
        PolySynth::new(&|| benihora_builder(), 1)
    } else if track == 9 {
        PolySynth::new(&noise_builder, 8)
    } else {
        // Box::new(PolySynth::new(&|| fm_synth_builder(program as u32), 8))
        PolySynth::new(&|| saw_builder(pitch.clone()), 8)
    };
    let gain = Param::with_value(1.0f64);
    let pan = Param::with_value(0.0f64);
    (synth, gain, pan, false, pitch_ctrl)
}

fn saw_builder(pitch: ProcOnceShare<f64, Controllable<f64, Param<f64, f64>>>) -> MyVoice {
    let (freq_param, mut freq_param_ctrl) = controllable_param(1.0);
    let (gain, mut gain_ctrl) = controllable_param(1.0);
    let (acc, mut acc_reset) = resetable_acc(Amp::new(freq_param, pitch));
    let saw = Add::new(acc, Constant::from(-0.5));
    let (env, mut env_on, mut env_off) = AdsrEnvelope::new(0.01, 0.5, 0.2, 0.3).build();
    let node = Amp::new(saw, Amp::new(env, gain));
    Voice(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            freq_param_ctrl
                .lock()
                .set_value_at_time(time, notenum_to_frequency(notenum as u32));
            gain_ctrl.lock().set_value_at_time(time, velocity);
            acc_reset(time, 0.5);
            env_on(time);
        }),
        Box::new(move |time, NoteOff(())| env_off(time)),
    )
}

fn noise_builder() -> MyVoice {
    let (gain, mut gain_ctrl) = controllable_param(1.0);
    let noise = Controllable::new(EventControlInplace::new(Noise::new()));
    let mut noise_ctrl = noise.controller();
    let (env, mut env_on, mut env_off) = ArEnvelope::new(0.01, 0.3).build();
    let node = Amp::new(noise, Amp::new(env, gain));
    Voice(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            noise_ctrl.lock().push_event(time, NoiseEvent::ResetReg);
            noise_ctrl.lock().push_event(
                time,
                NoiseEvent::OriginalFreq(notenum % 7, (15 * notenum as usize / 127) as u8),
            );
            gain_ctrl.lock().set_value_at_time(time, velocity * 0.25);
            env_on(time)
        }),
        Box::new(move |time, NoteOff(())| env_off(time)),
    )
}

fn fm_synth_builder(seed: u32) -> MyVoice {
    let (gain, mut gain_ctrl) = controllable_param(1.0);
    let synth = Controllable::new(rand_fm_synth(seed));
    let mut ctrl1 = synth.controller();
    let mut ctrl2 = synth.controller();
    let node = Amp::new(synth, gain);
    Voice(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            gain_ctrl.lock().set_value_at_time(time, velocity);
            ctrl1
                .lock()
                .note_on(time, notenum_to_frequency(notenum as u32));
        }),
        Box::new(move |time, NoteOff(())| {
            ctrl2.lock().note_off(time);
        }),
    )
}

fn benihora_builder() -> MyVoice {
    use corus::contrib::benihora::{make_noise_node, Benihora, BenihoraEvent};
    let benihora = Benihora::new(make_noise_node(), 2);
    let benihora = Controllable::new(EventControlInplace::new(benihora));
    let mut ctrl1 = benihora.controller();
    let mut ctrl2 = benihora.controller();
    ctrl2
        .lock()
        .push_event(0.0, BenihoraEvent::SetStatus(false, false));
    let benihora = corus::contrib::simple_comp::SimpleComp::new(
        benihora,
        Constant::from(0.2),
        Constant::from(0.5),
        Constant::from(1.4),
    );
    Voice(
        Box::new(Amp::new(benihora, Constant::from(1.5))) as Box<dyn Node<C1f64>>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            let time = time - 0.05;
            ctrl1
                .lock()
                .push_event(time, BenihoraEvent::SetStatus(true, false));
            ctrl1
                .lock()
                .push_event(time, BenihoraEvent::SetTenseness(velocity));
            ctrl1.lock().push_event(
                time,
                BenihoraEvent::MoveTangue(
                    perlin_noise(time * 2.3, time * 0.11, 0.0) * 19.0 + 22.0,
                    perlin_noise(time * 2.3, time * 0.11, 3.0) * 1.25 + 1.55,
                ),
            );
            ctrl1.lock().push_event(
                time,
                BenihoraEvent::SetFrequency(notenum_to_frequency(notenum as u32)),
            );
        }),
        Box::new(move |time, NoteOff(())| {
            let time = time - 0.05;
            ctrl2
                .lock()
                .push_event(time, BenihoraEvent::SetStatus(false, false));
        }),
    )
}

fn wavetable_builder(pitch: ProcOnceShare<f64, Controllable<f64, Param<f64, f64>>>) -> MyVoice {
    let wavetable = make_wavetable();
    let (freq_param, mut freq_param_ctrl) = controllable_param(1.0);
    let (gain, mut gain_ctrl) = controllable_param(1.0);
    let (acc, mut acc_reset) = resetable_acc(Amp::new(freq_param, pitch));
    let node = Map::new(acc, move |x| {
        wavetable[((x * wavetable.len() as f64) as usize).rem_euclid(wavetable.len())]
    });
    let (env, mut env_on, mut env_off) = AdsrEnvelope::new(0.01, 0.5, 0.2, 0.3).build();
    let node = Amp::new(node, Amp::new(env, gain));
    Voice(
        Box::new(node) as Box<dyn Node<C1f64>>,
        Box::new(move |time, NoteOn((notenum, velocity))| {
            freq_param_ctrl
                .lock()
                .set_value_at_time(time, notenum_to_frequency(notenum as u32));
            gain_ctrl.lock().set_value_at_time(time, velocity);
            acc_reset(time, 0.5);
            env_on(time);
        }),
        Box::new(move |time, NoteOff(())| env_off(time)),
    )
}

fn make_wavetable() -> Vec<f64> {
    let buf = vec![0.0, 0.2, 0.5, 0.3, 0.4, 0.3, 0.3, 0.1, 0.1, 0.1, -1.0];
    let acc = Accumulator::new(Constant::from(-1.0), 1.0);
    let node = Map::new(acc, move |f| {
        buf[((f * buf.len() as f64) as usize).rem_euclid(buf.len())]
    });
    let mut node = BiquadFilter::new(
        node,
        BiquadFilterParams::new(
            LowPass,
            Constant::from(500.0),
            Constant::from(0.0),
            Constant::from(1.0),
        ),
    );
    let buf: Vec<_> = ProcContext::new(1000).lock(&mut node).take(1000).collect();
    buf
}
