//! Headless NES 

pub(crate) mod bus;
pub mod cpu;
pub mod cartridge;
pub mod rom;

/// The number of bytes in a kibibyte (1 KiB).
pub(crate) const BYTES_ON_A_KIBIBYTE: usize = 1024;