pub mod iterator;
pub mod node;
pub mod poly_synth;
pub mod proc_context;
pub mod ring_buffer;

pub fn notenum_to_frequency(notenum: u32) -> f32 {
    440.0 * 2.0f32.powf((notenum as f32 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
