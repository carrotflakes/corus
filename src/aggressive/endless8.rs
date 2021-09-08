use std::marker::PhantomData;

use crate::{signal::Signal, Node, ProcContext};

// TODO cache

pub struct Endless8<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    node: A,
    length: f64,
    offset: f64,
    pub release: bool,
    _t: PhantomData<T>,
}

impl<T, A> Endless8<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    pub fn new(node: A, length: f64, offset: f64) -> Self {
        Endless8 {
            node,
            length,
            offset,
            release: false,
            _t: Default::default(),
        }
    }
}

impl<T, A> Node<T> for Endless8<T, A>
where
    T: Signal + Clone,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if !self.release && self.length < self.offset + ctx.time {
            self.offset -= self.length;
        }
        self.node.proc(&ProcContext {
            sample_rate: ctx.sample_rate,
            time: self.offset + ctx.time,
        })
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

// pub struct Endless8Cached<T, A>
// where
//     T: Signal + Clone,
//     A: Node<T>,
// {
//     node: A,
//     length: f64,
//     offset: f64,
//     pub release: bool,
//     _t: PhantomData<T>,
// }

// impl<T, A> Endless8Cached<T, A>
// where
//     T: Signal + Clone,
//     A: Node<T>,
// {
//     pub fn new(node: A, length: f64, offset: f64) -> Self {
//         Endless8Cached {
//             node,
//             length,
//             offset,
//             release: false,
//             _t: Default::default(),
//         }
//     }
// }

// impl<T, A> Node<T> for Endless8Cached<T, A>
// where
//     T: Signal + Clone,
//     A: Node<T>,
// {
//     #[inline]
//     fn proc(&mut self, ctx: &ProcContext) -> T {
//         if !self.release && self.length < self.offset + ctx.time {
//             self.offset -= self.length;
//         }
//         self.node.proc(&ProcContext {
//             sample_rate: ctx.sample_rate,
//             time: self.offset + ctx.time,
//         })
//     }

//     fn lock(&mut self, ctx: &ProcContext) {
//         self.node.lock(ctx);
//     }

//     fn unlock(&mut self) {
//         self.node.unlock();
//     }
// }
