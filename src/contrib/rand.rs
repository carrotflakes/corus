pub struct Rand(pub u32);

impl Rand {
    pub fn new(seed: u32) -> Self {
        Rand(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = 1103515245 * self.0 + 12345;
        self.0
    }

    /// returns 0.0 - 1.0
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f64 / std::u32::MAX as f64) as f32
    }

    /// returns 0.0 - 1.0
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / std::u32::MAX as f64
    }
}
