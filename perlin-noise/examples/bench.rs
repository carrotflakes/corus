use std::hint::black_box;

fn main() {
    let start = std::time::Instant::now();

    for _ in 0..10000000 {
        black_box(perlin_noise::perlin_noise([0.01, 0.2, 0.3]));
    }

    println!("{:?}", start.elapsed());
}
