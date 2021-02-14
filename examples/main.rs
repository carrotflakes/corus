use corus::node::ProcContext;

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
use corus::node::{self, Node};
use node::{add::Add, amp::Amp, constant::Constant, sine::Sine};

fn f<A: Node<f32>, B: Node<f32>, DA: AsMut<A>, DB: AsMut<B>>(frequency: DA, env: DB, gain: f32) -> Amp<f32, Sine<A, DA>, Amp<f32, Constant<f32>, B, Box<Constant<f32>>, DB>, Box<Sine<A, DA>>, Box<Amp<f32, Constant<f32>, B, Box<Constant<f32>>, DB>>>  {
    let sine = Box::new(Sine::new(frequency));
    let gain = Box::new(Constant::new(gain));
    let env = Box::new(node::amp::Amp::new(gain, env));
    node::amp::Amp::new(sine, env)
}

fn main() {
    let sample_rate = 44100;
    let env1 = {
        let param = node::param::Param::new();
        let mut ctrl = param.controller();
        ctrl.linera_ramp_to_value_at_time(0.5, 0.1);
        ctrl.linera_ramp_to_value_at_time(1.0, 0.05);
        ctrl.set_target_at_time(1.0, 0.0, 0.1);
        // ctrl.set_value_at_time(0.1, 0.1);
        // ctrl.set_value_at_time(0.2, 0.2);
        // ctrl.set_value_at_time(0.5, 0.5);
        // ctrl.set_value_at_time(1.0, 1.0);
        // ctrl.set_value_at_time(1.5, 0.8);
        // ctrl.set_value_at_time(2.0, 0.5);
        Box::new(param)
    };
    let env2 = Box::new(node::envelope::Envelope::new(0.1, 0.25, 0.5, 0.5, 2.0));
    let modu = f(Box::new(Constant::new(220.1)), env1, 3000.0);
    let add = Add::new(Constant::new(440.0), modu);
    let mut node = f(Box::new(add), env2, 1.0);
    let mut pc = ProcContext::new(sample_rate);
    let mut writer = Writer::new("output.wav");
    for _ in 0..sample_rate * 3 {
        let s = pc.sample(&mut node);
        writer.write(s, s);
    }
    writer.finish();
}
