use std::marker::PhantomData;

use super::{Node, ProcContext};

use crate::{Event, EventQueue};

pub struct ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    node: A,
    time: f64,
    value: T,
    pub(crate) lock_count: u32,
    bound_event_queue: Option<EventQueue>,
}

impl<T, A> ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    pub fn new(node: A) -> Self {
        ProcOnce {
            node,
            time: -1.0,
            value: Default::default(),
            lock_count: 0,
            bound_event_queue: None,
        }
    }

    pub(crate) fn get_ref(&self) -> &A {
        &self.node
    }

    // pub(crate) fn get_mut(&mut self) -> &mut DA {
    //     &mut self.node
    // }
}

impl<T, A> Node<T> for ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.time != ctx.current_time {
            self.time = ctx.current_time;
            self.value = self.node.proc(ctx);
        }
        self.value.clone()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.lock_count += 1;
        if self.lock_count == 1 {
            if let Some(eq) = &self.bound_event_queue {
                if eq != &ctx.event_queue {
                    panic!("this ProcOnce is shared by multiple contexts!");
                }
            } else {
                self.bound_event_queue = Some(ctx.event_queue.clone());
            }
            self.node.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        self.lock_count -= 1;
        if self.lock_count == 0 {
            self.node.unlock();
            // self.bound_event_queue = None;
        }
    }
}

pub struct ProcOnceEvent<T, A, E>
where
    T: 'static + Clone + Default,
    A: 'static + Node<T>,
    E: Event<Target = A>,
{
    event: E,
    _t: PhantomData<T>,
}

impl<T, A, E> ProcOnceEvent<T, A, E>
where
    T: 'static + Clone + Default,
    A: 'static + Node<T>,
    E: Event<Target = A>,
{
    pub fn new(event: E) -> Self {
        Self {
            event,
            _t: Default::default(),
        }
    }
}

impl<T, A, E> Event for ProcOnceEvent<T, A, E>
where
    T: 'static + Clone + Default,
    A: 'static + Node<T>,
    E: Event<Target = A>,
{
    type Target = A;

    fn dispatch(&self, time: f64, target: &mut Self::Target) {
        self.event.dispatch(time, target)
    }
}
