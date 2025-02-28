//! Holds the implementation of a NROM based cartridge.

use crate::cartridge::{Cartridge, CartridgeError};
use crate::rom::Rom;
use crate::BYTES_ON_A_KIBIBYTE;

/// Implementation for the cartridges that uses the NROM mapper chip.
///
/// # TODO
/// Currently support for [Family Basic](https://en.wikipedia.org/wiki/Family_BASIC)
/// is not available due to missing PRG RAM implementation.
pub(crate) struct Nrom {
    /// Dynamically holds the ROM of the cartridge.
    rom: Box<dyn Rom>,

    /// If the cartridge has 32KiB or 16KiB of PRG ROM size,
    /// the later enables mirroring of the ROM addresses.
    has_32_kibibytes_prg_rom_capacity: bool,
}

impl Nrom {
    /// Create a new NROM cartridge
    pub(crate) fn new<T: Rom + 'static>(has_32_kibibytes_prg_rom_capacity: bool, rom: T) -> Nrom {
        Nrom {
            rom: Box::new(rom),
            has_32_kibibytes_prg_rom_capacity,
        }
    }
}

impl Cartridge for Nrom {
    unsafe fn read(&self, address: u16) -> Result<u8, CartridgeError> {
        if address < 0x8000 {
            return Err(CartridgeError::CannotRead(
                "On a NROM memory mapper read operations below 0x800 are undefined behavior",
            ));
        }

        let address = address as usize - 0x8000;

        if self.has_32_kibibytes_prg_rom_capacity {
            return Ok(self.rom.read_prg_data(address));
        }

        Ok(self.rom.read_prg_data(address % (16 * BYTES_ON_A_KIBIBYTE)))
    }

    unsafe fn write(&mut self, _address: u16, _value: u8) -> Result<(), CartridgeError> {
        Err(CartridgeError::CannotWrite(
            "Write operations cannot be done with a NROM memory mapper",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_NROM_ADDRESS: u16 = 0x4020;
    const NROM_FIRST_ROM_BANK_ADDRESS: u16 = 0x8000;
    const NROM_SECOND_ROM_BANK_ADDRESS: u16 = 0xC000;

    struct MockRom;

    impl MockRom {
        const MOCK_VALUE_ON_LOWER_HALF: u8 = 0;
        const MOCK_VALUE_ON_HIGHER_HALF: u8 = 1;
    }

    impl Rom for MockRom {
        fn read_prg_data(&self, index: usize) -> u8 {
            if index >= BYTES_ON_A_KIBIBYTE {
                return MockRom::MOCK_VALUE_ON_HIGHER_HALF;
            }

            MockRom::MOCK_VALUE_ON_LOWER_HALF
        }
    }

    #[test]
    fn test_write_protection() {
        let mut nrom_cartridge = Nrom::new(true, MockRom {});

        unsafe {
            assert!(nrom_cartridge.write(INVALID_NROM_ADDRESS, 0).is_err());
            assert!(nrom_cartridge
                .write(NROM_FIRST_ROM_BANK_ADDRESS, 0)
                .is_err());
            assert!(nrom_cartridge
                .write(NROM_SECOND_ROM_BANK_ADDRESS, 0)
                .is_err());
        }
    }

    #[test]
    fn test_read_below_prg_protection() {
        let nrom_cartridge = Nrom::new(true, MockRom {});

        unsafe { assert!(nrom_cartridge.read(INVALID_NROM_ADDRESS).is_err()) }
    }

    #[test]
    fn test_read_on_32k() {
        let nrom_cartridge = Nrom::new(true, MockRom {});

        assert_eq!(
            unsafe { nrom_cartridge.read(NROM_FIRST_ROM_BANK_ADDRESS).unwrap() },
            MockRom::MOCK_VALUE_ON_LOWER_HALF
        );

        assert_eq!(
            unsafe { nrom_cartridge.read(NROM_SECOND_ROM_BANK_ADDRESS).unwrap() },
            MockRom::MOCK_VALUE_ON_HIGHER_HALF
        );
    }

    #[test]
    fn test_read_on_16k() {
        let nrom_cartridge = Nrom::new(false, MockRom {});

        assert_eq!(
            unsafe { nrom_cartridge.read(NROM_FIRST_ROM_BANK_ADDRESS).unwrap() },
            MockRom::MOCK_VALUE_ON_LOWER_HALF
        );

        assert_eq!(
            unsafe { nrom_cartridge.read(NROM_SECOND_ROM_BANK_ADDRESS).unwrap() },
            MockRom::MOCK_VALUE_ON_LOWER_HALF
        );
    }
}
