pub struct OnDemand<S> {
    buffer: Vec<S>,
    index: usize,
}

impl<S: Copy> OnDemand<S> {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            index: 0,
        }
    }

    pub fn process(&mut self, mut render: impl FnMut(&mut Vec<S>)) -> S {
        if self.index >= self.buffer.len() {
            self.buffer.clear();
            render(&mut self.buffer);
            self.index = 0;
        }
        let x = self.buffer[self.index];
        self.index += 1;
        x
    }
}
