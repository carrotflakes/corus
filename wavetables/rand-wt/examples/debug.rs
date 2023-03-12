fn main() {
    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);

    for _ in 0..3 {
        let wt = rand_wt::Config {
            least_depth: 0,
            variable_num: 0,
        }
        .generate(&mut rng);
        println!("{:?}", wt);
    }
    println!();

    for _ in 0..3 {
        let wt = rand_wt::Config {
            least_depth: 1,
            variable_num: 0,
        }
        .generate(&mut rng);
        println!("{:?}", wt);
    }
    println!();

    for _ in 0..3 {
        let wt = rand_wt::Config {
            least_depth: 2,
            variable_num: 0,
        }
        .generate(&mut rng);
        println!("{:?}", wt);
    }
    println!();
}
