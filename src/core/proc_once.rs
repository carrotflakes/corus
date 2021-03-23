use std::marker::PhantomData;

use super::{Node, ProcContext};

use crate::{EventListener, EventQueue};

pub struct ProcOnce<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    node: A,
    time: f64,
    value: A::Output,
    pub(crate) lock_count: u32,
    bound_event_queue: Option<EventQueue>,
}

impl<A> ProcOnce<A>
where
    A: Node,
    A::Output: Clone + Default,
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

impl<A> Node for ProcOnce<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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

pub struct ProcOnceEvent<A, E>
where
    A: Node,
    A::Output: Clone + Default,
{
    event: E,
    _t: PhantomData<A>,
}

impl<A, E> ProcOnceEvent<A, E>
where
    A: Node,
    A::Output: Clone + Default,
{
    pub fn new(event: E) -> Self {
        Self {
            event,
            _t: Default::default(),
        }
    }
}

impl<A, E> EventListener<ProcOnceEvent<A, E>> for ProcOnce<A>
where
    A: Node + EventListener<E>,
    A::Output: Clone + Default,
{
    #[inline]
    fn apply_event(&mut self, time: f64, event: &ProcOnceEvent<A, E>) {
        self.node.apply_event(time, &event.event)
    }
}
