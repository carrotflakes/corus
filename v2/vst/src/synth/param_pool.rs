use serde::{Deserialize, Serialize};

pub struct ParamPool {
    pub params: Vec<f64>,
}

impl ParamPool {
    pub fn new(producer_ids: &[ProducerId]) -> Self {
        Self { params: vec![0.0; producer_ids.len()] }
    }

    pub fn set(&mut self, producer_id: ProducerId, value: f64) {
        self.params[producer_id.0] = value;
    }

    pub fn get(&self, producer_id: ProducerId) -> f64 {
        self.params[producer_id.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProducerId(pub usize);

impl ProducerId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Consumer {
    pub producers: Vec<(f64, ProducerId)>,
}

impl Consumer {
    pub fn new() -> Self {
        Self {
            producers: Vec::new(),
        }
    }

    pub fn get(&self, pool: &ParamPool) -> f64 {
        self.producers
            .iter()
            .map(|(amount, producer_id)| amount * pool.get(*producer_id))
            .sum()
    }
}

#[test]
fn test() {
    let p1 = ProducerId::new(0);
    let p2 = ProducerId::new(1);
    let mut c1 = Consumer::new();
    c1.producers.push((1.0, p1));
    let pool = ParamPool::new(&[p1, p2]);
    dbg!(pool.get(p1));
}
