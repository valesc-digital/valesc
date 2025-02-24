use crate::{cartridge::Cartridge, rom::Rom, BYTES_ON_A_KIBIBYTE};

use super::CartridgeError;

struct Nrom {
    rom: Box<dyn Rom>,
    has_32_kibibytes_prg_rom_capacity: bool,
}

impl Nrom {
    pub(crate) fn new<T: Rom + 'static>(has_32_kibibytes_prg_rom_capacity: bool, rom: T) -> Nrom {
        Nrom {
            rom: Box::new(rom),
            has_32_kibibytes_prg_rom_capacity
        }
    }
}

impl Cartridge for Nrom {
    fn read(&self, address: u16) -> Result<u8, CartridgeError> {
        if address < 0x8000 {
            return Err(CartridgeError::CannotRead("On a NROM memory mapper read operations below 0x800 are undefined behavior"))
        }

        let address = address as usize - 0x8000;

        if self.has_32_kibibytes_prg_rom_capacity {
            return Ok(self.rom.read_prg_data(address));
        }

        Ok(self.rom.read_prg_data(address % (16 * BYTES_ON_A_KIBIBYTE)))
    }

    fn write(&mut self, _address: u16, _value: u8) -> Result<(), CartridgeError> {
        Err(CartridgeError::CannotWrite("Write operations cannot be done with a NROM memory mapper"))
    }
}