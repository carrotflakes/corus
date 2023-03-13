use std::collections::VecDeque;

pub struct EventQueue<T> {
    queue: VecDeque<(f64, T)>,
}

impl<T> EventQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, time: f64, event: T) {
        if let Some(index) = self.queue.iter().position(|(t, _)| *t > time) {
            self.queue.insert(index, (time, event));
        } else {
            self.queue.push_back((time, event));
        }
    }

    pub fn dispatch(&mut self, current_time: f64, mut f: impl FnMut(&mut Self, f64, T)) {
        let Some(x) = self.queue.front_mut() else {
            return;
        };
        if current_time < x.0 {
            return;
        }
        let x = self.queue.pop_front().unwrap();
        f(self, x.0, x.1);
        self.dispatch(current_time, f);
    }
}
