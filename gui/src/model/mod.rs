
pub struct Root {
    pub objects: Vec<Object>,
}

impl Root {
    pub fn new() -> Self { Self { objects: Vec::new() } }
}

#[derive(Debug)]
pub struct Object {
    pub id: usize,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub name: String,
    pub inputs: Vec<usize>,
}

impl Object {
    pub fn new(
        id: usize,
        position: (i32, i32),
        size: (u32, u32),
        name: String,
        inputs: Vec<usize>,
    ) -> Self {
        Self {
            id,
            position,
            size,
            name,
            inputs,
        }
    }
}
