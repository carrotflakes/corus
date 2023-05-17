use std::marker::PhantomData;

use crate::{
    signal::Signal, unsafe_wrapper::UnsafeWrapper, PackedEvent, ProcessContext, Producer,
};

#[deprecated]
pub struct PolySynth<P1, P2, A: Producer + NoteHandler<P1, P2>, ID: PartialEq + Default> {
    voices: Vec<VoiceContainer<P1, P2, A, ID>>,
}

struct VoiceContainer<P1, P2, A: Producer + NoteHandler<P1, P2>, ID: PartialEq + Default> {
    id: ID,
    voice: A,
    note_off_time: f64,
    _t: (PhantomData<P1>, PhantomData<P2>),
}

impl<P1, P2, A: Producer + NoteHandler<P1, P2>, ID: PartialEq + Default>
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

impl<
        P1: 'static,
        P2: 'static,
        A: 'static + Producer + NoteHandler<P1, P2>,
        ID: 'static + PartialEq + Default,
    > PolySynth<P1, P2, A, ID>
where
    A::Output: Signal,
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

    pub fn process(&mut self, ctx: &ProcessContext) -> A::Output {
        let mut v = A::Output::default();
        for voice in &mut self.voices {
            v = v + voice.voice.process(ctx);
        }
        v
    }
}

impl<
        P1: 'static + Send + Sync,
        P2: 'static + Send + Sync,
        A: 'static + Send + Sync + Producer + NoteHandler<P1, P2>,
        ID: 'static + Send + Sync + PartialEq + Default,
    > PolySynth<P1, P2, A, ID>
where
    A::Output: Signal,
{
    pub fn note_on_event(this: &UnsafeWrapper<Self>, id: ID, payload: P1) -> PackedEvent {
        let mut this = this.clone();
        Box::new(move |time: f64| {
            this.note_on(time, id, payload);
        })
    }

    pub fn note_off_event(this: &UnsafeWrapper<Self>, id: ID, payload: P2) -> PackedEvent {
        let mut this = this.clone();
        Box::new(move |time: f64| {
            this.note_off(time, id, payload);
        })
    }
}

pub struct Voice<A: Producer<Output = f64>, P1, P2> {
    node: A,
    note_on_fn: Box<dyn FnMut(f64, P1) + Send + Sync>,
    note_off_fn: Box<dyn FnMut(f64, P2) + Send + Sync>,
}

impl<A: Producer<Output = f64>, P1, P2> Voice<A, P1, P2> {
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

impl<A: Producer<Output = f64>, P1, P2> Producer for Voice<A, P1, P2> {
    type Output = f64;

    fn process(&mut self, ctx: &ProcessContext) -> Self::Output {
        self.node.process(ctx)
    }
}

impl<A: Producer<Output = f64>, P1, P2> NoteHandler<P1, P2> for Voice<A, P1, P2> {
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
