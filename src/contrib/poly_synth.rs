use crate::{
    core::Node,
    proc_context::ProcContext,
    signal::C1f64,
};

pub struct PolySynth<A: Node<Output = C1f64>> {
    voices: Vec<Voice<A>>,
    current: usize,
}

pub struct Voice<A: Node<Output = C1f64>> {
    last_notenum: u8,
    node: A,
    note_on: Box<dyn FnMut(f64, u8)>,
    note_off: Box<dyn FnMut(f64)>,
}

impl<A: Node<Output = C1f64>> Voice<A> {
    pub fn new(
        node: A,
        note_on: Box<dyn FnMut(f64, u8)>,
        note_off: Box<dyn FnMut(f64)>,
    ) -> Self {
        Self {
            last_notenum: 127,
            node,
            note_on,
            note_off,
        }
    }
}

impl<A: Node<Output = C1f64>> PolySynth<A> {
    pub fn new(voice_builder: &dyn Fn() -> Voice<A>, voice_num: usize) -> Self {
        Self {
            voices: (0..voice_num).map(|_| voice_builder()).collect(),
            current: 0,
        }
    }

    pub fn note_on(&mut self, time: f64, notenum: u8) {
        let current = self.current;
        let voice = &mut self.voices[current];
        voice.last_notenum = notenum;
        (voice.note_on)(time, notenum);
        self.current = (self.current + 1) % self.voices.len();
    }

    pub fn note_off(&mut self, time: f64, notenum: u8) {
        for voice in &mut self.voices {
            if voice.last_notenum == notenum {
                (voice.note_off)(time);
            }
        }
    }
}

impl<A: Node<Output = C1f64>> Node for PolySynth<A> {
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let mut v = Default::default();
        for voice in &mut self.voices {
            v = v + voice.node.proc(ctx);
        }
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        for voice in &mut self.voices {
            voice.node.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        for voice in &mut self.voices {
            voice.node.unlock();
        }
    }
}
