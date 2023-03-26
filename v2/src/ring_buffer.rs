#[derive(Clone)]
pub struct RingBuffer<T: Clone + Default> {
    buffer: Vec<T>,
    pos: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        Self {
            buffer: vec![Default::default(); size],
            pos: size - 1,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn push(&mut self, value: T) {
        self.pos = (self.pos + 1) % self.buffer.len();
        self.buffer[self.pos] = value;
    }

    #[inline]
    pub fn get(&self, index: usize) -> T {
        let size = self.buffer.len();
        if size < index {
            panic!("RingBuffer out of bounds")
        }
        let i = (size + self.pos - index) % size;
        self.buffer[i].clone()
    }

    #[inline]
    pub fn fast_resize(&mut self, size: usize) {
        self.buffer.resize(size, Default::default());
        self.pos = self.pos % size;
    }
}

#[test]
fn test() {
    let mut b = RingBuffer::new(4);

    assert_eq!(b.size(), 4);
    b.push(1);
    b.push(2);
    b.push(3);
    b.push(4);
    assert_eq!(b.get(0), 4);
    assert_eq!(b.get(1), 3);
    assert_eq!(b.get(2), 2);
    assert_eq!(b.get(3), 1);
    b.push(5);
    assert_eq!(b.get(0), 5);
    assert_eq!(b.get(1), 4);
    assert_eq!(b.get(2), 3);
    assert_eq!(b.get(3), 2);
}
