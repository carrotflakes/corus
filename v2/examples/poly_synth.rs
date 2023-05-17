use corus_v2::{
    event_queue::EventQueue,
    nodes::{envelope::Envelope, phase::Phase, voice_manager::VoiceManager},
    signal::IntoStereo,
    ProcessContext,
};

fn main() {
    let mut ctx = ProcessContext::new(44100.0);
    let mut event_queue = EventQueue::new();
    let mut synth = PolySynth::new();

    event_queue.push(0.25, (true, 60));
    event_queue.push(0.5, (false, 60));
    event_queue.push(0.5, (true, 63));
    event_queue.push(0.75, (false, 63));
    event_queue.push(0.75, (true, 67));
    event_queue.push(1.0, (false, 67));
    event_queue.push(1.25, (true, 60));
    event_queue.push(2.0, (false, 60));
    event_queue.push(1.26, (true, 63));
    event_queue.push(2.0, (false, 63));
    event_queue.push(1.27, (true, 67));
    event_queue.push(2.0, (false, 67));

    let name = "poly_synth.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 3 {
        event_queue.dispatch(ctx.current_time(), |_eq, time, event| {
            synth.handle_event(time, event);
        });

        let x = synth.process(&ctx);
        let x = x.into_stereo_with_pan(0.0);

        writer
            .write_sample((x[0] * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((x[1] * std::i16::MAX as f64) as i16)
            .unwrap();

        ctx.next();
    }
    writer.finalize().unwrap();
}

struct PolySynth {
    voices: VoiceManager<u8, Voice>,
    envelope: Envelope,
}

impl PolySynth {
    fn new() -> Self {
        Self {
            voices: VoiceManager::new(Voice::new, 8),
            envelope: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
        }
    }

    fn handle_event(&mut self, time: f64, event: (bool, u8)) {
        let notenum = event.1;
        if event.0 {
            let mut voice = self.voices.note_on(notenum);
            voice.start_time = time;
            voice.end_time = std::f64::INFINITY;
            voice.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
            voice.amplitude = 1.0;
        } else {
            if let Some(mut voice) = self.voices.note_off(notenum) {
                voice.end_time = time;
            }
        }
    }

    fn process(&mut self, ctx: &ProcessContext) -> f64 {
        self.voices
            .iter_mut()
            .map(|v| {
                let phase = v.phase.process(&ctx, v.frequency);
                let x = wavetables::primitives::square(phase);
                x * v.amplitude
                    * self
                        .envelope
                        .compute(ctx.current_time() - v.start_time, v.end_time - v.start_time)
            })
            .sum::<f64>()
            * 0.1
    }
}

struct Voice {
    phase: Phase<f64>,
    frequency: f64,
    amplitude: f64,
    start_time: f64,
    end_time: f64,
}

impl Voice {
    fn new() -> Self {
        Self {
            phase: Phase::new(),
            frequency: 1.0,
            amplitude: 0.0,
            start_time: 0.0,
            end_time: 0.0,
        }
    }
}
