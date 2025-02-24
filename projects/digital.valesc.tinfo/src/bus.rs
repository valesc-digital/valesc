use thiserror::Error;
use rand::prelude::*;

use crate::{cartridge::{Cartridge, CartridgeError}, BYTES_ON_A_KIBIBYTE};

const CPU_RAM_WITH_MIRRORING_START_ADDRESS: u16 = 0x0000;
const CPU_RAM_WITH_MIRRORING_END_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTERS_WITH_MIRRORING_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTERS_WITH_MIRRORING_END_ADDRESS: u16 = 0x3FFF;

const APU_AND_IO_REGISTERS_START_ADDRESS: u16 = 0x4000;
const APU_AND_IO_REGISTERS_END_ADDRESS: u16 = 0x4017;

const APU_AND_IO_CPU_TEST_MODE_REGISTERS_START_ADDRESS: u16 = 0x4018;
const APU_AND_IO_CPU_TEST_MODE_REGISTERS_END_ADDRESS: u16 = 0x401F;

const CARTRIDGE_CONTROLLED_REGION_START_ADDRESS: u16 = 0x4020;
const CARTRIDGE_CONTROLLED_REGION_END_ADDRESS: u16 = 0xFFFF;

pub(crate) struct Bus {
    cpu_ram: [u8; 2 * BYTES_ON_A_KIBIBYTE],
    pub cartridge: Box<dyn Cartridge>,
}

#[derive(Error, Debug)]
enum BusError {
    #[error("Unable to read from the shared memory address space: {0}")]
    CannotRead(&'static str),

    #[error("Unable to write to the shared memory address space: {0}")]
    CannotWrite(&'static str),

    #[error("Unable to access to the cartridge: {0}")]
    CartridgeError(#[from] CartridgeError),
}

impl Bus {
    pub(crate) fn new<T: Cartridge + 'static>(cartridge: T) -> Bus {
        // The CPU RAM should be randomized to emulate the undefined state of the bits on startup,
        // used on some games as a pseudo RNG

        let mut rng = rand::rng();
        let cpu_ram: Vec<u8> = rng.random_iter().take(2 * BYTES_ON_A_KIBIBYTE).collect();

        Bus {
            cpu_ram: cpu_ram.try_into().unwrap(),
            cartridge: Box::new(cartridge),
        }
    }

    pub(crate) fn read(&self, address: u16) -> Result<u8, BusError> {
        match address {
            CPU_RAM_WITH_MIRRORING_START_ADDRESS..=CPU_RAM_WITH_MIRRORING_END_ADDRESS => {
                // Remove everything past the first 11 bits, mirroring the memory in the process
                let masked_adress = address & 0b00000111_11111111;

                Ok(self.cpu_ram[masked_adress as usize])
            },

            PPU_REGISTERS_WITH_MIRRORING_START_ADDRESS..=PPU_REGISTERS_WITH_MIRRORING_END_ADDRESS => {
                // It's only needed to check the first three bits of the address to get the number of the PPU register to access 
                todo!("PPU registers have not been implemented yet")
            },

            APU_AND_IO_REGISTERS_START_ADDRESS..=APU_AND_IO_REGISTERS_END_ADDRESS => {
                todo!("APU and IO registers have not been implemented yet")
            },

            APU_AND_IO_CPU_TEST_MODE_REGISTERS_START_ADDRESS..=APU_AND_IO_CPU_TEST_MODE_REGISTERS_END_ADDRESS => {
                todo!("APU and IO special registers when the CPU is in Test Mode have not been implemented yet")
            }

            CARTRIDGE_CONTROLLED_REGION_START_ADDRESS..=CARTRIDGE_CONTROLLED_REGION_END_ADDRESS => self.cartridge.read(address).map_err(BusError::CartridgeError),
        }
    }

    pub(crate) fn write(&mut self, address: u16, value: u8) -> Result<(), BusError> {
        //println!("Wrote @ {address:#02X}: {value:#02X}");
        
        match address {
            CPU_RAM_WITH_MIRRORING_START_ADDRESS..=CPU_RAM_WITH_MIRRORING_END_ADDRESS => {
                // Remove everything past the first 11 bits
                let masked_adress = address & 0b00000111_11111111;

                self.cpu_ram[masked_adress as usize] = value;

                Ok(())
            },

            PPU_REGISTERS_WITH_MIRRORING_START_ADDRESS..=PPU_REGISTERS_WITH_MIRRORING_END_ADDRESS => {
                // It's only needed to check the first three bits of the address to get the number of the PPU register to access 
                todo!("PPU registers have not been implemented yet")
            },

            APU_AND_IO_REGISTERS_START_ADDRESS..=APU_AND_IO_REGISTERS_END_ADDRESS => {
                todo!("APU and IO registers have not been implemented yet")
            },

            APU_AND_IO_CPU_TEST_MODE_REGISTERS_START_ADDRESS..=APU_AND_IO_CPU_TEST_MODE_REGISTERS_END_ADDRESS => {
                todo!("APU and IO special registers when the CPU is in Test Mode have not been implemented yet")
            }

            CARTRIDGE_CONTROLLED_REGION_START_ADDRESS..=CARTRIDGE_CONTROLLED_REGION_END_ADDRESS => self.cartridge.write(address, value).map_err(BusError::CartridgeError),
        }
    }

    /*
    fn read_mapper_controlled_region(&self, address: u16) -> Result<u8, BusError> {
        match self.rom.mapper() {
            Mapper::Nrom { has_32_kibibytes_prg_rom_capacity } => {
                if address < 0x8000 {
                    return Err(BusError::CannotRead("On a NROM memory mapper read operations below 0x800 are undefined behavior"))
                }

                let address = address as usize - 0x8000;

                if has_32_kibibytes_prg_rom_capacity {
                    return Ok(self.rom.read_prg(address));
                }

                Ok(self.rom.read_prg(address % (16 * BYTES_ON_A_KIBIBYTE)))
            }
        }
    }

    fn write_mapper_controlled_region(&mut self, _address: u16, _value: u8) -> Result<(), BusError> {
        match self.rom.mapper() {
            Mapper::Nrom { has_32_kibibytes_prg_rom_capacity: _ } => {
                Err(BusError::CannotWrite("Write operations cannot be done with a NROM memory mapper"))
            }
        }
    }
    */
}

#[cfg(test)]
mod tests {
    use super::*;

    const CPU_RAM_WITH_MIRRORING_START_ADDRESS: u16 = 0x0000;
    const CPU_RAM_SIZE: usize = 2 * BYTES_ON_A_KIBIBYTE;

    const MAPPER_CONTROLLED_REGION_START_ADDRESS: u16 = 0x4020;

    // Test to do:
    // - CPU RAM read/write
    // - CPU RAM MIRRORS read/write (all of them)
    // - Mappers read/write:
    //      - NROM
    // Probably more...

    // - Check if write in RAM is preserved
    // - Check if read mirrors of RAM are working
    // - Check if NROM memory is plain accessed on 32K mode.
    // - Check if NROM memory is mirrored on 16K mode.
    // - Check if NROM memory is write protected (on 16K and 32K modes).
    // - Check if IO access is redirected to the proper mapper. 

    // TODO:
    // - Split Mapper/Bus implementation.
    // - Add no random mode of the bus.

    /*
    #[test]
    fn test_nrom_mapper_fail_on_illegal_memory_address() {
        let rom = MockNromRom::new(false);
        let bus = Bus::new(rom);

        let read_value = bus.read(MAPPER_CONTROLLED_REGION_START_ADDRESS);

        assert!(read_value.is_err());
    }
    */

    /*
    #[test]
    fn test_check_ram_io() {
        let mut rom = MockRom::new();

        rom.expect_mapper()
            .return_const(Mapper::Nrom { has_32_kibibytes_prg_rom_capacity: false });

        rom.expect_read_prg()
            .withf(|index| *index == 123123)
            .return_const(0);

        let bus = Bus::new(rom);

        // FIRST WRITE A WELL KNOWN VALUE (DISABLE RANDOM RAM?) AND THEN CHECK THREE TIMES

        bus.read(CPU_RAM_WITH_MIRRORING_START_ADDRESS).unwrap();
        bus.read(CPU_RAM_WITH_MIRRORING_START_ADDRESS + CPU_RAM_SIZE as u16).unwrap();
        bus.read(CPU_RAM_WITH_MIRRORING_START_ADDRESS + CPU_RAM_SIZE as u16 * 2).unwrap();
        bus.read(CPU_RAM_WITH_MIRRORING_START_ADDRESS + CPU_RAM_SIZE as u16 * 3).unwrap();
    }

    #[test]
    fn test_check_rom_nrom_mapper_16k_prg_read() {
        let mut rom = MockNromRom::new(false);

        bus.read().unwrap();
    }

    #[test]
    fn test_check_rom_nrom_mapper_32k_prg_read() {

    }

    #[test]
    fn test_check_rom_nrom_mapper_16k_prg_write() {
    }

    #[test]
    fn test_check_rom_nrom_mapper_32k_prg_write() {
    }
        */

}