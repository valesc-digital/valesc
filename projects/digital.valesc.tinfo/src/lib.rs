//! Headless NES

pub(crate) mod bus;
pub mod cartridge;
pub mod cpu;
pub mod rom;

/// The number of bytes in a kibibyte (1 KiB).
pub(crate) const BYTES_ON_A_KIBIBYTE: usize = 1024;

#[inline(always)]
/// Build a NES CPU bus address given two bytes.
pub(crate) fn build_address(lower_byte: u8, upper_byte: u8) -> u16 {
    (lower_byte as u16) | ((upper_byte as u16) << 8)
}

/// Extension methods for the [u16] type.
trait U16Ex {
    /// Get the least significant byte of the [u16].
    fn get_lower_byte(&self) -> u8;

    /// Get the most significant byte of the [u16].
    fn get_upper_byte(&self) -> u8;
}

impl U16Ex for u16 {
    #[inline(always)]
    fn get_lower_byte(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    #[inline(always)]
    fn get_upper_byte(&self) -> u8 {
        ((self & 0xFF00) >> 8) as u8
    }
}

#[cfg(test)]
mod tests {
    use crate::{build_address, U16Ex};

    #[test]
    fn test_build_address() {
        assert_eq!(build_address(0xAB, 0xDC), 0xDCAB);
    }

    #[test]
    fn test_u16_get_lower_byte() {
        assert_eq!(0xFF00_u16.get_lower_byte(), 0x00);
    }

    #[test]
    fn test_u16_get_upper_byte() {
        assert_eq!(0xFF00_u16.get_upper_byte(), 0xFF);
    }
}