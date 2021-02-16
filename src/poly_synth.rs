use std::{cell::RefCell, rc::Rc};

use crate::{
    node::{param::Param, Node},
    proc_context::ProcContext,
};

pub struct PolySynth<A: Node<f32>>(Rc<RefCell<PolySynthBody<A>>>);

pub struct PolySynthBody<A: Node<f32>> {
    voices: Vec<Voice<A>>,
    current: usize,
}

pub struct Voice<A: Node<f32>> {
    frequency: f32,
    frequency_param: Param,
    node: A,
    note_on: Box<dyn FnMut(f64)>,
    note_off: Box<dyn FnMut(f64)>,
}

impl<A: Node<f32>> Voice<A> {
    pub fn new(
        frequency_param: Param,
        node: A,
        note_on: Box<dyn FnMut(f64)>,
        note_off: Box<dyn FnMut(f64)>,
    ) -> Self {
        Self {
            frequency: 1.0,
            frequency_param,
            node,
            note_on,
            note_off,
        }
    }
}

impl<A: Node<f32>> PolySynth<A> {
    pub fn new(voice_builder: &dyn Fn() -> Voice<A>) -> Self {
        Self(Rc::new(RefCell::new(PolySynthBody {
            voices: (0..10).map(|_| voice_builder()).collect(),
            current: 0,
        })))
    }

    pub fn controller(&self) -> Self {
        PolySynth(self.0.clone())
    }

    pub fn note_on(&mut self, time: f64, frequency: f32) {
        let mut body = self.0.borrow_mut();
        let current = body.current;
        let voice = &mut body.voices[current];
        voice.frequency = frequency;
        voice.frequency_param.set_value_at_time(time, frequency);
        (voice.note_on)(time);
        body.current = (body.current + 1) % body.voices.len();
    }

    pub fn note_off(&mut self, time: f64, frequency: f32) {
        let mut body = self.0.borrow_mut();
        for voice in &mut body.voices {
            if voice.frequency == frequency {
                (voice.note_off)(time);
            }
        }
    }
}

impl<A: Node<f32>> Node<f32> for PolySynth<A> {
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        let mut body = self.0.borrow_mut();
        body.voices
            .iter_mut()
            .map(|voice| voice.node.proc(ctx))
            .sum()
    }
}

impl<A: Node<f32>> AsMut<Self> for PolySynth<A> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
