use std::marker::PhantomData;

use crate::{
    node::Node,
    proc_context::ProcContext,
    signal::C1f64,
};

pub struct PolySynth<A: Node<C1f64> + ?Sized, DA: AsMut<A>> {
    voices: Vec<Voice<A, DA>>,
    current: usize,
}

pub struct Voice<A: Node<C1f64> + ?Sized, DA: AsMut<A>> {
    last_notenum: u8,
    node: DA,
    note_on: Box<dyn FnMut(f64, u8)>,
    note_off: Box<dyn FnMut(f64)>,
    _a: PhantomData<A>,
}

impl<A: Node<C1f64> + ?Sized, DA: AsMut<A>> Voice<A, DA> {
    pub fn new(
        node: DA,
        note_on: Box<dyn FnMut(f64, u8)>,
        note_off: Box<dyn FnMut(f64)>,
    ) -> Self {
        Self {
            last_notenum: 127,
            node,
            note_on,
            note_off,
            _a: Default::default(),
        }
    }
}

impl<A: Node<C1f64> + ?Sized, DA: AsMut<A>> PolySynth<A, DA> {
    pub fn new(voice_builder: &dyn Fn() -> Voice<A, DA>, voice_num: usize) -> Self {
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

impl<A: Node<C1f64> + ?Sized, DA: AsMut<A>> Node<C1f64> for PolySynth<A, DA> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let mut v = Default::default();
        for voice in &mut self.voices {
            v = v + voice.node.as_mut().proc(ctx);
        }
        v
    }

    fn lock(&mut self) {
        for voice in &mut self.voices {
            voice.node.as_mut().lock();
        }
    }

    fn unlock(&mut self) {
        for voice in &mut self.voices {
            voice.node.as_mut().unlock();
        }
    }
}

impl<A: Node<C1f64> + ?Sized, DA: AsMut<A>> AsMut<Self> for PolySynth<A, DA> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
