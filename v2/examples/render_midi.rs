use corus_v2::{
    event_queue::EventQueue,
    nodes::{envelope::Envelope, phase::Phase, voice_manager::VoiceManager},
    signal::{IntoStereo, StereoF64},
    ProcessContext,
};

fn main() {
    let file = std::env::args()
        .skip(1)
        .next()
        .unwrap_or("../youkoso.mid".to_string());

    let mut ctx = ProcessContext::new(44100.0);
    let mut event_queue = EventQueue::new();
    let mut synths: Vec<_> = (0..16).map(|_| PolySynth::new()).collect();

    // Load MIDI file
    let time = {
        let mut time = 0.0;
        let data = std::fs::read(&file).unwrap();
        let events = ezmid::parse(&data);
        for e in ezmid::Dispatcher::new(events) {
            time = e.time;
            event_queue.push(e.time, e.event);
        }
        time + 1.0
    };

    let name = "render_midi.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..(ctx.sample_rate() * time) as usize {
        event_queue.dispatch(ctx.current_time(), |_eq, time, event| {
            let synth = &mut synths[event.channel as usize];
            synth.handle_event(time, event.body);
        });

        let mut x = StereoF64::default();
        for synth in &mut synths {
            x = x + synth.process(&ctx);
        }
        x = x * 0.1;

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
    volume: f64,
    pan: f64,
    pitch: f64,
}

impl PolySynth {
    fn new() -> Self {
        Self {
            voices: VoiceManager::new(Voice::new, 8),
            envelope: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
            volume: 0.8,
            pan: 0.0,
            pitch: 1.0,
        }
    }

    fn handle_event(&mut self, time: f64, event: ezmid::EventBody) {
        #[allow(unused_variables)]
        match event {
            ezmid::EventBody::NoteOn {
                notenum, velocity, ..
            } => {
                let mut voice = self.voices.note_on(notenum);
                voice.start_time = time;
                voice.end_time = std::f64::INFINITY;
                voice.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
                voice.amplitude = velocity as f64;
            }
            ezmid::EventBody::NoteOff {
                notenum, velocity, ..
            } => {
                if let Some(mut voice) = self.voices.note_off(notenum) {
                    voice.end_time = time;
                }
            }
            ezmid::EventBody::Volume { volume, .. } => {
                self.volume = volume as f64;
            }
            ezmid::EventBody::Pan { pan, .. } => {
                self.pan = pan as f64;
            }
            ezmid::EventBody::Modulation {
                modulation,
                raw_modulation,
            } => {}
            ezmid::EventBody::Expression {
                expression,
                raw_expression,
            } => {}
            ezmid::EventBody::PitchBend { bend, .. } => {
                self.pitch = 2.0f64.powf(bend as f64 / 12.0);
            }
            ezmid::EventBody::ProgramChange { program } => {}
            ezmid::EventBody::Tempo { .. } => {}
        }
    }

    fn process(&mut self, ctx: &ProcessContext) -> StereoF64 {
        self.voices
            .iter_mut()
            .map(|v| {
                let phase = v.phase.process(&ctx, v.frequency * self.pitch);
                let x = wavetables::primitives::square(phase);
                x * v.amplitude
                    * self
                        .envelope
                        .compute(ctx.current_time() - v.start_time, v.end_time - v.start_time)
            })
            .sum::<f64>()
            .into_stereo_with_pan(self.pan)
            * self.volume
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
