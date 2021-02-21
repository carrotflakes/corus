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
    node::{amp::Amp, constant::Constant, controllable::Controllable, mix::Mix, Node},
    notenum_to_frequency,
    proc_context::ProcContext,
    signal::{C1f32, C2f32},
};
const SAMPLE_RATE: usize = 44100;

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("./youkoso.mid".to_string());

    let builder = || {
        let (freq_param, mut freq_param_ctrl) = controllable_param(1.0);
        let (acc, mut acc_reset) = resetable_acc(freq_param);
        let saw = corus::node::add::Add::new(acc, Constant::from(-0.5));
        let (env, mut env_on, env_off) = AdsrEnvelope {
            a: 0.01,
            d: 0.5,
            s: 0.2,
            r: 0.3,
        }
        .build();
        let node = Amp::new(saw, env);
        Voice::new(
            Box::new(node) as Box<dyn Node<C1f32>>,
            Box::new(move |time, notenum| {
                freq_param_ctrl
                    .lock()
                    .set_value_at_time(time, notenum_to_frequency(notenum as u32));
                acc_reset(time, 0.5);
                env_on(time);
            }),
            Box::new(env_off),
        )
    };
    let builder2 = || {
        let noise = Controllable::new(EventControll::new(Noise::new()));
        let mut noise_ctrl = noise.controller();
        let (env, mut env_on, env_off) = ArEnvelope { a: 0.01, r: 0.3 }.build();
        let node = Amp::new(noise, Amp::new(env, Constant::from(0.25)));
        Voice::new(
            Box::new(node) as Box<dyn Node<C1f32>>,
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
    };
    let mut tracks: Vec<_> = (0..16)
        .map(|i| {
            let synth = if i == 9 {
                Box::new(PolySynth::new(&builder2, 8))
                    as Box<PolySynth<dyn Node<C1f32>, Box<dyn Node<C1f32>>>>
            } else {
                // Box::new(PolySynth::new(&|| builder3(i + 10), 8))
                Box::new(PolySynth::new(&builder, 8))
                    as Box<PolySynth<dyn Node<C1f32>, Box<dyn Node<C1f32>>>>
            };
            let (gain, gain_ctrl) = controllable_param(1.0);
            let (pan, pan_ctrl) = controllable_param(0.0);
            (synth, gain, pan, gain_ctrl, pan_ctrl)
        })
        .collect();

    let time = {
        let data = std::fs::read(&file).unwrap();
        let events = ezmid::parse(&data);
        let mut time = 0.0;
        for e in ezmid::Dispatcher::new(events) {
            time = e.time;

            // mute drum part
            // if e.event.channel == 9 {
            //     continue;
            // }

            let track = &mut tracks[e.event.channel as usize];
            match e.event.body {
                ezmid::EventBody::NoteOn {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.0.note_on(e.time, notenum);
                }
                ezmid::EventBody::NoteOff {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    track.0.note_off(e.time, notenum);
                }
                ezmid::EventBody::Volume { volume, .. } => {
                    track.3.lock().set_value_at_time(e.time, volume);
                }
                ezmid::EventBody::Pan { pan, .. } => {
                    track.4.lock().set_value_at_time(e.time, pan);
                }
                ezmid::EventBody::PitchBend {
                    bend: _,
                    raw_bend: _,
                } => {}
                ezmid::EventBody::Tempo { tempo: _ } => {}
            }
        }
        time + 1.0
    };

    let mix = Mix::new(
        tracks
            .into_iter()
            .map(|t| Box::new(amp_pan(t.0, t.1, t.2)) as Box<dyn Node<C2f32>>)
            .collect(),
    );
    let node = Amp::new(mix, Constant::new(C2f32([0.25, 0.25])));
    let node = delay_fx(node, SAMPLE_RATE as usize, 0.3, 0.3);

    let file = format!("{}.wav", file[..file.len() - 4].to_string());
    write_to_file(file.as_str(), SAMPLE_RATE, time, node);
    println!("saved {:?}", &file);
}

fn builder3(seed: u32) -> Voice<dyn Node<C1f32>, Box<dyn Node<C1f32>>> {
    let synth = Controllable::new(rand_fm_synth(seed));
    let mut ctrl1 = synth.controller();
    let mut ctrl2 = synth.controller();
    Voice::new(
        Box::new(synth) as Box<dyn Node<C1f32>>,
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

pub fn write_to_file<N: Node<C2f32>, DN: AsMut<N>>(name: &str, sample_rate: usize, len: f64, mut node: DN) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    let pc = ProcContext::new(sample_rate as u64);
    let start = std::time::Instant::now();
    node.as_mut().lock();
    for s in pc.into_iter(&mut node).take((sample_rate as f64 * len).ceil() as usize) {
        writer
            .write_sample((s.0[0] * std::i16::MAX as f32) as i16)
            .unwrap();
        writer
            .write_sample((s.0[1] * std::i16::MAX as f32) as i16)
            .unwrap();
    }
    node.as_mut().unlock();
    writer.finalize().unwrap();
    println!("{:?} elapsed", start.elapsed());
}
