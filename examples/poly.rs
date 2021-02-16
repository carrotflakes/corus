use corus::{
    node::{amp::Amp, constant::Constant, param::Param, sine::Sine},
    notenum_to_frequency,
    poly_synth::{PolySynth, Voice},
    proc_context::ProcContext,
};

fn main() {
    let sample_rate = 44100;

    let builder = || {
        let freq_param = Param::new();
        let sine = Sine::new(freq_param.controller());
        let env = Param::new();
        let node = Amp::new(sine, env.controller());
        Voice::new(
            freq_param,
            node,
            Box::new({
                let mut env = env.controller();
                move |time| {
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + 0.01, 1.0);
                    env.exponential_ramp_to_value_at_time(time + 0.2, 0.5);
                }
            }),
            Box::new({
                let mut env = env.controller();
                move |time| {
                    env.cancel_and_hold_at_time(time);
                    env.set_target_at_time(time, 0.0, 0.1);
                }
            }),
        )
    };
    let mut synth = PolySynth::new(&builder);
    synth.note_on(0.2, notenum_to_frequency(60));
    synth.note_off(0.4, notenum_to_frequency(60));
    synth.note_on(0.4, notenum_to_frequency(64));
    synth.note_off(0.6, notenum_to_frequency(64));
    synth.note_on(0.6, notenum_to_frequency(67));
    synth.note_off(0.8, notenum_to_frequency(67));
    synth.note_on(0.8, notenum_to_frequency(64));
    synth.note_off(1.0, notenum_to_frequency(64));
    synth.note_on(1.0, notenum_to_frequency(60));
    synth.note_off(1.6, notenum_to_frequency(60));
    synth.note_on(1.1, notenum_to_frequency(64));
    synth.note_off(1.6, notenum_to_frequency(64));
    synth.note_on(1.2, notenum_to_frequency(67));
    synth.note_off(1.6, notenum_to_frequency(67));
    synth.note_on(1.3, notenum_to_frequency(70));
    synth.note_off(1.6, notenum_to_frequency(70));


    let node = Amp::new(synth, Constant::new(0.1));

    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("poly.wav");
    for s in pc.into_iter(node).take(sample_rate as usize * 3) {
        writer.write(s, s);
    }
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
