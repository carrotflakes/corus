use crate::{event_controll::Event, node::Node, proc_context::ProcContext, signal::C1f32};

pub struct Noise {
    pub freq: u32,
    pub short_freq: bool,
    pub reg: u16,
    pub output: u16,
    steps: f32,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            freq: Self::compute_freq(0, 0),
            short_freq: true,
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

impl Node<C1f32> for Noise {
    fn proc(&mut self, ctx: &ProcContext) -> C1f32 {
        self.steps += self.freq as f32 / ctx.sample_rate as f32;
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
        C1f32([self.output as f32 * 2.0 - 1.0])
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl AsMut<Self> for Noise {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

pub enum NoiseEvent {
    OriginalFreq(f64, u8, u8),
    Freq(f64, u32),
    ShortFreq(f64, bool),
    ResetReg(f64),
}

impl Event<C1f32> for NoiseEvent {
    type Node = Noise;

    fn get_time(&self) -> f64 {
        match self {
            NoiseEvent::OriginalFreq(time, _, _) => *time,
            NoiseEvent::Freq(time, _) => *time,
            NoiseEvent::ShortFreq(time, _) => *time,
            NoiseEvent::ResetReg(time) => *time,
        }
    }

    fn dispatch(&self, node: &mut Self::Node) {
        match self {
            NoiseEvent::OriginalFreq(_, f1, f2) => {
                node.freq = Noise::compute_freq(*f1, *f2);
            }
            NoiseEvent::Freq(_, freq) => {
                node.freq = *freq;
            }
            NoiseEvent::ShortFreq(_, short_freq) => {
                node.short_freq = *short_freq;
            }
            NoiseEvent::ResetReg(_) => {
                node.reg = 1;
            }
        }
    }
}
