pub mod contrib;
pub mod sample_iterator;
pub mod node;
pub mod proc_context;
pub mod ring_buffer;
pub mod signal;

pub fn notenum_to_frequency(notenum: u32) -> f32 {
    440.0 * 2.0f32.powf((notenum as f32 - 69.0) / 12.0)
}
