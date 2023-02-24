#[derive(Clone, Default)]
pub struct State {
    pub xs: [f64; 2],
    pub ys: [f64; 2],
}

impl State {
    pub fn new() -> Self {
        Self {
            xs: [0.0; 2],
            ys: [0.0; 2],
        }
    }

    pub fn process(&mut self, params: &[f64; 6], x: f64) -> f64 {
        let [a0, a1, a2, b0, b1, b2] = *params;
        let y =
            (b0 * x + b1 * self.xs[0] + b2 * self.xs[1] - a1 * self.ys[0] - a2 * self.ys[1]) / a0;
        self.xs[1] = self.xs[0];
        self.xs[0] = x;
        self.ys[1] = self.ys[0];
        self.ys[0] = y;
        y
    }
}
