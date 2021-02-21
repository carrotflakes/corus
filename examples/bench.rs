use corus::{node::{
        add::Add, amp::Amp, ring_buffer_record::RingBufferRecord, ring_buffer_playback::RingBufferPlayback, constant::Constant,
        controllable::Controllable, mix::Mix, param::Param, placeholder::Placeholder,
        proc_once_share::ProcOnceShare, sine::Sine, Node,
    }, proc_context::ProcContext, signal::C1f32};

fn main() {
    let sample_rate = 44100;

    let mut nodes = Vec::new();
    let modulator =
        ProcOnceShare::new(Amp::new(Sine::new(Constant::from(4.0)), Constant::from(20.0)));
    for i in 0..100 {
        let freq = Controllable::new(Param::new());
        let mut freq_ctrl = freq.controller();
        let f = 440.0 * (i + 1) as f32;
        freq_ctrl.lock().set_value_at_time(0.0, f);
        freq_ctrl.lock().linear_ramp_to_value_at_time(1.0, f * 2.0);
        freq_ctrl.lock().exponential_ramp_to_value_at_time(2.0, f);
        nodes.push(Box::new(Amp::new(
            Sine::new(Add::new(freq, modulator.clone())),
            Constant::from(1.0 / (i + 1) as f32),
        )) as Box<dyn Node<C1f32>>);
    }

    let mix = Mix::new(nodes);

    let mix = {
        let mut p = Placeholder::new();
        let mut ps = p.setter();
        let buffer = ProcOnceShare::new(RingBufferRecord::new(p, sample_rate as usize));
        unsafe {
            ps.set(Box::new(Add::new(
                mix,
                Amp::new(
                    RingBufferPlayback::new(Constant::from(0.5), buffer.clone()),
                    Constant::from(0.5),
                ),
            )) as Box<dyn Node<C1f32>>);
        }
        buffer
    };

    let mut node = Amp::new(mix, Constant::from(0.1));

    let pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("bench.wav");
    let start = std::time::Instant::now();
    node.lock();
    for s in pc.into_iter(&mut node).take(sample_rate as usize * 4) {
        writer.write(s.0[0], s.0[0]);
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
