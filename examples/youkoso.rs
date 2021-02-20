use corus::{node::{Node, accumulator::Accumulator, amp::Amp, constant::Constant, controllable::Controllable, pan::Pan, param::Param}, notenum_to_frequency, poly_synth::{PolySynth, Voice}, proc_context::ProcContext, signal::C1f32};

fn main() {
    let sample_rate = 44100;

    let builder = || {
        let freq_param = Controllable::new(Param::new());
        let freq_param_ctrl = freq_param.controller();
        let acc = Controllable::new(Accumulator::new(freq_param, C1f32::from(1.0)));
        let mut acc_ctrl = acc.controller();
        let saw = corus::node::add::Add::new(acc, Constant::from(-0.5));
        let env = Controllable::new(Param::new());
        let env_ctrl = env.controller();
        let node = Amp::new(saw, env);
        Voice::new(
            freq_param_ctrl,
            node,
            Box::new({
                let mut env_ctrl = env_ctrl.clone();
                move |time| {
                    acc_ctrl.lock().set_value_at_time(time, C1f32::from(0.5));
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + 0.01, 1.0);
                    env.exponential_ramp_to_value_at_time(time + 0.2, 0.5);
                }
            }),
            Box::new({
                let mut env_ctrl = env_ctrl.clone();
                move |time| {
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_target_at_time(time, 0.0, 0.1);
                }
            }),
        )
    };
    let mut synth = PolySynth::new(&builder, 32);

    let time = {
        let data = std::fs::read("youkoso.mid").unwrap();
        let events = ezmid::parse(&data);
        let mut time = 0.0;
        for e in ezmid::Dispatcher::new(events) {
            time = e.time;
            match e.event.body {
                ezmid::EventBody::NoteOn {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    synth.note_on(e.time, notenum_to_frequency(notenum as u32));
                }
                ezmid::EventBody::NoteOff {
                    notenum,
                    velocity: _,
                    raw_velocity: _,
                } => {
                    synth.note_off(e.time, notenum_to_frequency(notenum as u32));
                }
                _ => {}
            }
        }
        time + 1.0
    };

    let mut node = Amp::new(synth, Constant::new(C1f32::from(0.1)));
    let mut node = Pan::new(node, Constant::from(-0.5));

    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("youkoso.wav");
    let start = std::time::Instant::now();
    node.lock();
    for s in pc.into_iter(&mut node).take((sample_rate as f64 * time) as usize) {
        writer.write(s.0[0], s.0[1]);
    }
    node.unlock();
    println!("{:?} elapsed", start.elapsed());
    writer.finish();
}

pub struct Writer(hound::WavWriter<std::io::BufWriter<std::fs::File>>);

impl Writer {
    pub fn new(name: &str) -> Self {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        Writer(hound::WavWriter::create(name, spec).unwrap())
    }

    pub fn write(&mut self, sample1: f32, sample2: f32) {
        self.0
            .write_sample((sample1 * std::i16::MAX as f32) as i16)
            .unwrap();
        self.0
            .write_sample((sample2 * std::i16::MAX as f32) as i16)
            .unwrap();
    }

    pub fn finish(self) {
        self.0.finalize().unwrap();
    }
}
