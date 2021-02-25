pub mod contrib;
pub mod node;
pub mod proc_context;
pub mod ring_buffer;
pub mod signal;

pub fn notenum_to_frequency(notenum: u32) -> f64 {
    440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0)
}
