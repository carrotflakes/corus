mod waveform;

pub struct Benihora {
    glottis: Glottis,
    tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    proc_num: usize,
}

impl Benihora {
    pub fn new(proc_num: usize) -> Self {
        Self {
            glottis: Glottis::new(),
            tract: Tract::new(),
            block_time: 0.04,
            block_updated_time: 0.0,
            proc_num,
        }
    }

    pub fn process(&mut self, x: f64) {}
}

pub struct Glottis {
    pub frequency: LinearInterp,
    pub tenseness: LinearInterp,

    pub intensity: f64,
    pub loudness: f64,
    pub breath: bool,
    pub glottis_close: bool,
}

impl Glottis {
    pub fn new(
        frequency: f64,
        tenseness: f64,
        intensity: f64,
        loudness: f64,
        breath: bool,
        glottis_close: bool,
    ) -> Self {
        Self {
            frequency: LinearInterp::new(0.01, frequency),
            tenseness: LinearInterp::new(0.01, tenseness),
            intensity,
            loudness,
            breath,
            glottis_close,
        }
    }
}

/// Param-like struct that smoothly changes its value.
pub struct LinearInterp {
    y0: f64,
    y1: f64,
    time: f64,
    start_time: Option<f64>,
}

impl LinearInterp {
    pub fn new(time: f64, y: f64) -> Self {
        Self {
            y0: y,
            y1: y,
            time,
            start_time: None,
        }
    }

    pub fn set(&mut self, y: f64) {
        self.y0 = self.y1;
        self.y1 = y;
        self.start_time = None;
    }

    pub fn process(&mut self, ctx: &corus_v2::ProcessContext) -> f64 {
        let x = ctx.current_time();
        if self.start_time.is_none() {
            self.start_time = Some(x);
        }
        let t = x - self.start_time.unwrap();
        if t < self.time {
            self.y0 + (self.y1 - self.y0) * t / self.time
        } else {
            self.y1
        }
    }
}

pub struct OnDemand {
    buffer: Vec<f64>,
    index: usize,
}

impl OnDemand {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            index: 0,
        }
    }

    pub fn process(&mut self, mut render: impl FnMut(&mut Vec<f64>)) -> f64 {
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
