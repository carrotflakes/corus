pub mod contrib;
pub mod core;
mod event_dispatcher;
mod node;
mod proc_context;
pub mod ring_buffer;
pub mod signal;
pub mod time;

pub use event_dispatcher::*;
pub use node::*;
pub use proc_context::*;

pub fn notenum_to_frequency(notenum: u32) -> f64 {
    440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0)
}

pub fn db_to_amp(db: f64) -> f64 {
    10.0f64.powf(db / 20.0)
}
