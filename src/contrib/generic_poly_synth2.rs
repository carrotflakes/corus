use std::marker::PhantomData;

use crate::{core::Node, proc_context::ProcContext, signal::C1f64};

pub struct PolySynth<P1, P2, A: Node<Output = C1f64>, ID: PartialEq + Default> {
    voices: Vec<VoiceContainer<P1, P2, A, ID>>,
    current: usize,
}

struct VoiceContainer<P1, P2, A: Node<Output = C1f64>, ID: PartialEq + Default> {
    id: ID,
    voice: A,
    _t: (PhantomData<P1>, PhantomData<P2>),
}

impl<P1, P2, A: Node<Output = C1f64>, ID: PartialEq + Default> VoiceContainer<P1, P2, A, ID> {
    pub fn new(node: A) -> Self {
        Self {
            id: Default::default(),
            voice: node,
            _t: Default::default(),
        }
    }
}

impl<P1, P2, A: Node<Output = C1f64>, ID: PartialEq + Default> PolySynth<P1, P2, A, ID> {
    pub fn new(voice_builder: &mut dyn FnMut() -> A, voice_num: usize) -> Self {
        Self {
            voices: (0..voice_num)
                .map(|_| VoiceContainer::new(voice_builder()))
                .collect(),
            current: 0,
        }
    }
}

impl<
        P1: 'static + Clone,
        P2: 'static + Clone,
        A: Node<Output = C1f64>,
        ID: PartialEq + Default,
    > Node for PolySynth<P1, P2, A, ID>
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

pub struct NoteOn<P>(pub P);
pub struct NoteOff<P>(pub P);

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

impl<A: 'static + Node<Output = f64>, P1: 'static + Clone, P2: 'static + Clone>
    crate::EventListener<NoteOn<P1>> for Voice<A, P1, P2>
{
    fn apply_event(&mut self, time: f64, event: &NoteOn<P1>) {
        self.1(time, event.0.clone());
    }
}
impl<A: 'static + Node<Output = f64>, P1: 'static + Clone, P2: 'static + Clone>
    crate::EventListener<NoteOff<P2>> for Voice<A, P1, P2>
{
    fn apply_event(&mut self, time: f64, event: &NoteOff<P2>) {
        self.2(time, event.0.clone());
    }
}

pub enum Event<P1, P2> {
    NoteOn(P1),
    NoteOff(P2),
}

pub trait Triggerable<Payload> {
    fn bang(&mut self, time: f64, payload: Payload);
}

impl<
        P1: 'static + Clone,
        P2: 'static,
        A: Node<Output = C1f64> + crate::EventListener<NoteOn<P1>>,
        ID: 'static + Clone + PartialEq + Default,
    > crate::EventListener<NoteOn<(ID, P1)>> for PolySynth<P1, P2, A, ID>
{
    fn apply_event(&mut self, time: f64, event: &NoteOn<(ID, P1)>) {
        let id = event.0 .0.clone();
        let payload = event.0 .1.clone();
        let current = self.current;
        let voice = &mut self.voices[current];
        voice.id = id;
        voice.voice.apply_event(time, &NoteOn(payload));
        self.current = (self.current + 1) % self.voices.len();
    }
}

impl<
        P1: 'static,
        P2: 'static + Clone,
        A: Node<Output = C1f64> + crate::EventListener<NoteOff<P2>>,
        ID: 'static + Clone + PartialEq + Default,
    > crate::EventListener<NoteOff<(ID, P2)>> for PolySynth<P1, P2, A, ID>
{
    fn apply_event(&mut self, time: f64, event: &NoteOff<(ID, P2)>) {
        let id = event.0 .0.clone();
        let payload = event.0 .1.clone();
        for voice in &mut self.voices {
            if voice.id == id {
                voice.voice.apply_event(time, &NoteOff(payload));
                voice.id = Default::default();
                return;
            }
        }
    }
}

impl<
        P1: 'static + Clone,
        P2: 'static + Clone,
        A: Node<Output = C1f64> + crate::EventListener<NoteOn<P1>> + crate::EventListener<NoteOff<P2>>,
        ID: 'static + Clone + PartialEq + Default,
    > crate::EventListener<Event<(ID, P1), (ID, P2)>> for PolySynth<P1, P2, A, ID>
{
    fn apply_event(&mut self, time: f64, event: &Event<(ID, P1), (ID, P2)>) {
        match event {
            Event::NoteOn(payload) => self.apply_event(time, &NoteOn(payload.clone())),
            Event::NoteOff(payload) => self.apply_event(time, &NoteOff(payload.clone())),
        }
    }
}
