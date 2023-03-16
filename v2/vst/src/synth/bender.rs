#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bender {
    None,
    Quadratic,
    Cubic,
    Sin,
    Cos,
}

impl Bender {
    pub fn process(&self, level: f64, x: f64) -> f64 {
        match self {
            Bender::None => x,
            Bender::Quadratic => wavetables::bend::quadratic_bender(level)(x),
            Bender::Cubic => wavetables::bend::cubic_bender(level)(x),
            Bender::Sin => wavetables::bend::sin_bender(level)(x),
            Bender::Cos => wavetables::bend::cos_bender(level)(x),
        }
    }

    pub fn level_range(&self) -> std::ops::Range<f64> {
        match self {
            Bender::None => 0.0..0.0,
            Bender::Quadratic => -1.0..1.0,
            Bender::Cubic => -0.5..4.0,
            Bender::Sin => -1.0..1.0,
            Bender::Cos => 0.0..1.0,
        }
    }
}
