/// The period is 2147483646
pub struct Rand(u32);

impl Rand {
    pub fn new(seed: u32) -> Self {
        assert!(seed != 0);
        Rand(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.overflowing_mul(48271).0 % ((1 << 31) - 1);
        self.0
    }

    /// returns 0.0 - 1.0
    pub fn next_f32(&mut self) -> f32 {
        ((self.next_u32() << 1) as f64 / std::u32::MAX as f64) as f32
    }

    /// returns 0.0 - 1.0
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u32() << 1) as f64 / std::u32::MAX as f64
    }
}
