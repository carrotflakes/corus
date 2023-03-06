use std::marker::PhantomData;

use crate::{core::Node, proc_context::ProcContext, signal::C1f64};

pub struct PolySynth<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default>
{
    voices: Vec<VoiceContainer<P1, P2, A, ID>>,
    current: usize,
}

struct VoiceContainer<
    P1,
    P2,
    A: Node<Output = C1f64> + NoteHandler<P1, P2>,
    ID: PartialEq + Default,
> {
    id: ID,
    voice: A,
    _t: (PhantomData<P1>, PhantomData<P2>),
}

impl<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default>
    VoiceContainer<P1, P2, A, ID>
{
    pub fn new(node: A) -> Self {
        Self {
            id: Default::default(),
            voice: node,
            _t: Default::default(),
        }
    }
}

impl<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default>
    PolySynth<P1, P2, A, ID>
{
    pub fn new(mut voice_builder: impl FnMut() -> A, voice_num: usize) -> Self {
        Self {
            voices: (0..voice_num)
                .map(|_| VoiceContainer::new(voice_builder()))
                .collect(),
            current: 0,
        }
    }

    pub fn note_on(&mut self, time: f64, id: ID, payload: P1) {
        let current = self.current;
        let voice = &mut self.voices[current];
        voice.id = id;
        voice.voice.note_on(time, payload);
        self.current = (self.current + 1) % self.voices.len();
    }

    pub fn note_off(&mut self, time: f64, id: ID, payload: P2) {
        for voice in &mut self.voices {
            if voice.id == id {
                voice.voice.note_off(time, payload);
                voice.id = Default::default();
                return;
            }
        }
    }
}

impl<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default> Node
    for PolySynth<P1, P2, A, ID>
{
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let mut v = Default::default();
        for voice in &mut self.voices {
            v = v + voice.voice.proc(ctx);
        }
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        for voice in &mut self.voices {
            voice.voice.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        for voice in &mut self.voices {
            voice.voice.unlock();
        }
    }
}

pub struct Voice<A: Node<Output = C1f64>, P1, P2>(
    pub A,
    pub Box<dyn FnMut(f64, P1) + Send + Sync>,
    pub Box<dyn FnMut(f64, P2) + Send + Sync>,
);

impl<A: Node<Output = C1f64>, P1, P2> Node for Voice<A, P1, P2> {
    type Output = C1f64;

    fn proc(&mut self, ctx: &crate::ProcContext) -> Self::Output {
        self.0.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.0.lock(ctx);
    }

    fn unlock(&mut self) {
        self.0.unlock();
    }
}

impl<A: Node<Output = f64>, P1, P2> NoteHandler<P1, P2> for Voice<A, P1, P2> {
    fn note_on(&mut self, time: f64, payload: P1) {
        self.1(time, payload);
    }

    fn note_off(&mut self, time: f64, payload: P2) {
        self.2(time, payload);
    }
}

pub trait NoteHandler<P1, P2> {
    fn note_on(&mut self, time: f64, payload: P1);
    fn note_off(&mut self, time: f64, payload: P2);
}
