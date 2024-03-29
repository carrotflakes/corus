use crate::{EventListener, core::Node, proc_context::ProcContext, signal::{C1f64, Mono}};

pub struct Noise {
    pub freq: u32,
    pub short_freq: bool,
    pub reg: u16,
    pub output: u16,
    steps: f64,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            freq: Self::compute_freq(0, 0),
            short_freq: false,
            reg: 0xffff,
            output: 1,
            steps: 0.0,
        }
    }

    /// freq1: count, 0-7
    /// freq2: octave: 0-15
    pub fn compute_freq(freq1: u8, freq2: u8) -> u32 {
        let mut f = 524288;
        f /= [1, 2, 4, 6, 8, 10, 12, 14][freq1 as usize];
        f = f >> freq2;
        f
    }
}

impl Node for Noise {
    type Output = C1f64;

    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        self.steps += self.freq as f64 / ctx.sample_rate as f64;
        for _ in 0..self.steps as usize {
            if self.reg == 0 {
                self.reg = 1;
            }
            self.reg = self
                .reg
                .overflowing_add(
                    self.reg
                        + (((self.reg >> (if self.short_freq { 6 } else { 14 }))
                            ^ (self.reg >> (if self.short_freq { 5 } else { 13 })))
                            & 1),
                )
                .0;
            self.output ^= self.reg & 1;
        }
        self.steps = self.steps.fract();
        C1f64::from_m(self.output as f64 * 2.0 - 1.0)
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub enum NoiseEvent {
    OriginalFreq(u8, u8),
    Freq(u32),
    ShortFreq(bool),
    ResetReg,
}

impl EventListener<NoiseEvent> for Noise {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &NoiseEvent) {
        match event {
            NoiseEvent::OriginalFreq(f1, f2) => {
                self.freq = Noise::compute_freq(*f1, *f2);
            }
            NoiseEvent::Freq(freq) => {
                self.freq = *freq;
            }
            NoiseEvent::ShortFreq(short_freq) => {
                self.short_freq = *short_freq;
            }
            NoiseEvent::ResetReg => {
                self.reg = 1;
            }
        }
    }
}
