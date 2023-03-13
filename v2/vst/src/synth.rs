use corus_v2::{
    nodes::{
        biquad_filter::BiquadFilter, effects::DelayFx, envelope::Envelope, sine::Sine,
        unison::Unison, voice_manager::VoiceManager,
    },
    signal::{IntoStereo, SignalExt, StereoF64},
    ProccessContext,
};

pub struct MySynth {
    voices: VoiceManager<u8, MyVoice>,
    gain: f64,
    pan: f64,
    pub pitch: f64,
    pub frequency: f64,
    pub q: f64,
    mod_level: f64,
    mod_sine: Sine,
    filter: BiquadFilter<2, StereoF64>,
    delay_fx: DelayFx<StereoF64>,
}

impl MySynth {
    pub fn new() -> Self {
        let voices = VoiceManager::new(|| MyVoice::new(0), 8);
        Self {
            voices,
            gain: 1.0,
            pan: 0.0,
            pitch: 1.0,
            frequency: 1000.0,
            q: 1.0,
            mod_level: 0.0,
            mod_sine: Sine::new(),
            filter: BiquadFilter::new(),
            delay_fx: DelayFx::new(48000),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext) -> StereoF64 {
        let modu = self.mod_sine.process(ctx, 3.0) * self.mod_level;
        let pitch = self.pitch * modu.exp2();
        let mut x = StereoF64::default();
        for voice in self.voices.iter_mut() {
            x = x.add(voice.process(ctx, pitch));
        }
        x = self.filter.process(ctx, self.frequency, self.q, x);
        x = self.delay_fx.process(ctx, x, 0.5, 0.3);
        (x.mul(self.gain.into_stereo())).into_stereo_with_pan(self.pan)
    }

    pub fn handle_event(&mut self, event: MyEvent, time: f64) {
        match event {
            MyEvent::NoteOn(notenum, velocity) => {
                let v = self.voices.note_on(notenum);
                v.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
                // v.unison.reset();
                v.gain = velocity;
                v.env.note_on(time);
                v.filter_env.note_on(time);
            }
            MyEvent::NoteOff(notenum) => {
                if let Some(v) = self.voices.note_off(notenum) {
                    v.env.note_off(time);
                    v.filter_env.note_off(time);
                }
            }
            MyEvent::ProgramChange(program) => {
                self.voices = VoiceManager::new(move || MyVoice::new(program as u64), 8);
            }
            MyEvent::SetModLevel(level) => {
                self.mod_level = level;
            }
        }
    }
}

pub enum MyEvent {
    NoteOn(u8, f64),
    NoteOff(u8),
    ProgramChange(u8),
    SetModLevel(f64),
}

pub struct MyVoice {
    wt: Box<dyn Fn(f64) -> f64 + Send + Sync + 'static>,
    unison: Unison,
    frequency: f64,
    gain: f64,
    env: Envelope,
    filter_env: Envelope,
    filter: BiquadFilter<2, StereoF64>,
}

impl MyVoice {
    pub fn new(seed: u64) -> Self {
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(seed);
        let wt = rand_wt::Config {
            least_depth: 1,
            variable_num: 0,
        }
        .generate(&mut rng)
        .build();
        Self {
            wt,
            unison: Unison::new(3),
            frequency: 440.0,
            gain: 0.0,
            env: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
            filter_env: Envelope::new(&[(0.01, 1.0, -1.0), (0.4, 0.3, 1.0)], 0.3, 1.0),
            filter: BiquadFilter::new(),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext, pitch: f64) -> StereoF64 {
        let env = self.env.process(ctx);
        let gain = self.gain * env;
        let x = self
            .unison
            .process(ctx, self.frequency * pitch, 0.01, 0.9, |phase| {
                (self.wt)(phase)
            });
        let filter_env = self.filter_env.process(ctx);
        let x = self
            .filter
            .process(ctx, filter_env * 10000.0 + 20.0, 1.5, x);
        x.mul(gain.into_stereo())
    }
}
