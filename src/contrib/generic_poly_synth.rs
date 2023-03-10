use std::marker::PhantomData;

use crate::{core::Node, proc_context::ProcContext, signal::C1f64};

pub struct PolySynth<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default>
{
    voices: Vec<VoiceContainer<P1, P2, A, ID>>,
}

struct VoiceContainer<
    P1,
    P2,
    A: Node<Output = C1f64> + NoteHandler<P1, P2>,
    ID: PartialEq + Default,
> {
    id: ID,
    voice: A,
    note_off_time: f64,
    _t: (PhantomData<P1>, PhantomData<P2>),
}

impl<P1, P2, A: Node<Output = C1f64> + NoteHandler<P1, P2>, ID: PartialEq + Default>
    VoiceContainer<P1, P2, A, ID>
{
    pub fn new(node: A) -> Self {
        Self {
            id: Default::default(),
            voice: node,
            note_off_time: 0.0,
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
        }
    }

    pub fn note_on(&mut self, time: f64, id: ID, payload: P1) {
        let i = self.next_voice_index();
        let voice = &mut self.voices[i];
        voice.id = id;
        voice.voice.note_on(time, payload);
        voice.note_off_time = f64::INFINITY;
    }

    pub fn note_off(&mut self, time: f64, id: ID, payload: P2) {
        for voice in &mut self.voices {
            if voice.id == id {
                voice.voice.note_off(time, payload);
                voice.id = Default::default();
                voice.note_off_time = time;
                return;
            }
        }
    }

    pub fn get_voice_mut(&mut self, id: ID) -> Option<&mut A> {
        for voice in &mut self.voices {
            if voice.id == id {
                return Some(&mut voice.voice);
            }
        }
        None
    }

    fn next_voice_index(&mut self) -> usize {
        self.voices
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.note_off_time.total_cmp(&b.1.note_off_time))
            .unwrap()
            .0
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

pub struct Voice<A: Node<Output = C1f64>, P1, P2> {
    node: A,
    note_on_fn: Box<dyn FnMut(f64, P1) + Send + Sync>,
    note_off_fn: Box<dyn FnMut(f64, P2) + Send + Sync>,
}

impl<A: Node<Output = C1f64>, P1, P2> Voice<A, P1, P2> {
    pub fn new(
        node: A,
        note_on_fn: Box<dyn FnMut(f64, P1) + Send + Sync>,
        note_off_fn: Box<dyn FnMut(f64, P2) + Send + Sync>,
    ) -> Self {
        Self {
            node,
            note_on_fn,
            note_off_fn,
        }
    }
}

impl<A: Node<Output = C1f64>, P1, P2> Node for Voice<A, P1, P2> {
    type Output = C1f64;

    fn proc(&mut self, ctx: &crate::ProcContext) -> Self::Output {
        self.node.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl<A: Node<Output = f64>, P1, P2> NoteHandler<P1, P2> for Voice<A, P1, P2> {
    fn note_on(&mut self, time: f64, payload: P1) {
        (self.note_on_fn)(time, payload);
    }

    fn note_off(&mut self, time: f64, payload: P2) {
        (self.note_off_fn)(time, payload);
    }
}

pub trait NoteHandler<P1, P2> {
    fn note_on(&mut self, time: f64, payload: P1);
    fn note_off(&mut self, time: f64, payload: P2);
}
