mod audio;
mod component;
mod context;
mod f;
mod fm;
mod rand_fm;
mod interface;
mod framework;
mod model;

fn main() {
    f::f::<framework::Ui>();
    // rand_fm::rand_fm();
    // fm::fm();
}
