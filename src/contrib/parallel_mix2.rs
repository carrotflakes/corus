use std::{
    ops::Add,
    sync::{Arc, Condvar, Mutex},
    thread,
};

use crate::{time::Sample, EventQueue, Node, ProcContext};

const CHUNK_SIZE: usize = 1024;

pub struct ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    threads: Vec<thread::JoinHandle<()>>,
    request: Arc<Mutex<Request<A>>>,
    start: Arc<(Mutex<bool>, Condvar)>,
    samples: Vec<A::Output>,
    samples_i: usize,
    samples_i_max: usize,
}

impl<A> ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    pub fn new(nodes: Vec<A>, thread_num: usize) -> Self {
        let start = Arc::new((Mutex::new(false), Condvar::new()));
        let lines = nodes
            .into_iter()
            .map(|node| Line {
                line: Some(LineInner {
                    node,
                    samples: vec![Default::default(); CHUNK_SIZE],
                    context: None,
                }),
                samples: vec![Default::default(); CHUNK_SIZE],
                current: 0,
                overrun: false,
            })
            .collect();
        let request = Arc::new(Mutex::new(Request {
            lines,
            size: 0,
            current: 0,
            end: false,
        }));
        let mut threads = vec![];
        for _ in 0..thread_num {
            let start = start.clone();
            let request = request.clone();
            threads.push(thread::spawn(move || loop {
                let (lock, cvar) = &*start;
                drop(
                    cvar.wait_while(lock.lock().unwrap(), |started| !*started)
                        .unwrap(),
                );
                let size;
                let line = {
                    let mut request = request.lock().unwrap();
                    if request.end {
                        return;
                    }
                    size = request.size;
                    let current_min = request.lines.iter().map(|line| line.current).min().unwrap();
                    let mut a = None;
                    let mut b = None;
                    for (i, line) in request.lines.iter().enumerate() {
                        if line.line.is_some() && !(size <= line.current || line.overrun) {
                            if line.current == current_min {
                                a = Some(i);
                                break;
                            } else if line.current == current_min + CHUNK_SIZE {
                                b = Some(i);
                            }
                        }
                    }
                    a.or(b).map(|i| {
                        let line = &mut request.lines[i];
                        (i, line.line.take().unwrap(), line.current)
                    })
                };
                if let Some((i, mut line_inner, current)) = line {
                    let proc_length = (size - current).min(CHUNK_SIZE) as u64;
                    for (i, x) in line_inner
                        .context
                        .as_mut()
                        .unwrap()
                        .lock(&mut line_inner.node, Sample(proc_length))
                        .enumerate()
                    {
                        line_inner.samples[i] = x;
                    }
                    let mut request = request.lock().unwrap();
                    let current = request.current;
                    let line = &mut request.lines[i];
                    if line.current == current {
                        std::mem::swap(&mut line.samples, &mut line_inner.samples);
                    } else {
                        line.overrun = true;
                    }
                    line.current += proc_length as usize;
                    line.line = Some(line_inner);
                } else {
                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }));
        }
        Self {
            threads,
            request,
            start,
            samples: vec![Default::default(); CHUNK_SIZE],
            samples_i: CHUNK_SIZE,
            samples_i_max: CHUNK_SIZE,
        }
    }
}

impl<A> Node for ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> Self::Output {
        if self.samples_i == self.samples_i_max {
            loop {
                {
                    let mut request = self.request.lock().unwrap();
                    if request
                        .lines
                        .iter()
                        .all(|line| request.current < line.current)
                    {
                        self.samples.fill(Default::default());
                        for line in request.lines.iter_mut() {
                            for i in 0..self.samples.len() {
                                self.samples[i] = self.samples[i].clone() + line.samples[i].clone();
                            }
                            if line.overrun {
                                std::mem::swap(
                                    &mut line.samples,
                                    &mut line.line.as_mut().unwrap().samples,
                                );
                                line.overrun = false;
                            }
                        }
                        self.samples_i_max =
                            request.size.saturating_sub(request.current).min(CHUNK_SIZE);
                        self.samples_i = 0;
                        request.current += self.samples_i_max;
                        break;
                    }
                }
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        let s = self.samples[self.samples_i].clone();
        self.samples_i += 1;
        s
    }

    fn lock(&mut self, ctx: &ProcContext) {
        let mut request = self.request.lock().unwrap();
        for line in request.lines.iter_mut() {
            let mut line_inner = line.line.as_mut().unwrap();
            let mut context = ctx.clone();
            if let Some(old_context) = line_inner.context.take() {
                context.event_queue = old_context.event_queue;
            } else {
                context.event_queue = EventQueue::new();
            }
            line_inner.node.lock(&context);
            line_inner.context = Some(context);
        }
        request.size = ctx.rest_proc_samples as usize;
        request.current = 0;
        self.samples_i = CHUNK_SIZE;
        self.samples_i_max = CHUNK_SIZE;
        let (lock, cvar) = &*self.start;
        *lock.lock().unwrap() = true;
        cvar.notify_all();
    }

    fn unlock(&mut self) {
        *self.start.0.lock().unwrap() = false; // ?
        let mut request = self.request.lock().unwrap();
        for line in request.lines.iter_mut() {
            let line_inner = line.line.as_mut().unwrap();
            line_inner.node.unlock();
        }
    }
}

impl<A> Drop for ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    fn drop(&mut self) {
        self.request.lock().unwrap().end = true;
        let (lock, cvar) = &*self.start;
        *lock.lock().unwrap() = true;
        cvar.notify_all(); // ???

        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

pub struct LineInner<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    node: A,
    samples: Vec<A::Output>,
    context: Option<ProcContext>,
}

pub struct Line<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    line: Option<LineInner<A>>,
    samples: Vec<A::Output>,
    current: usize,
    overrun: bool,
}

struct Request<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    lines: Vec<Line<A>>,
    size: usize,
    current: usize,
    end: bool,
}

#[test]
fn test() {
    let nodes = vec![
        crate::core::sine::Sine::new(crate::core::var::Var::from(1.0)),
        crate::core::sine::Sine::new(crate::core::var::Var::from(2.0)),
        crate::core::sine::Sine::new(crate::core::var::Var::from(3.0)),
        crate::core::sine::Sine::new(crate::core::var::Var::from(4.0)),
    ];
    let mut mix = ParallelMix::new(nodes, 3);
    let mut ctx = ProcContext::new(10);
    let a = ctx.lock(&mut mix, Sample(15)).collect::<Vec<_>>();
    dbg!(&a[0..10]);
}
