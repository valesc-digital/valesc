use thiserror::Error;

use crate::BYTES_ON_A_KIBIBYTE;

const CPU_RAM_WITH_MIRRORING_START_ADDRESS: u16 = 0x0000;
const CPU_RAM_WITH_MIRRORING_END_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTERS_WITH_MIRRORING_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTERS_WITH_MIRRORING_END_ADDRESS: u16 = 0x3FFF;

const APU_AND_IO_REGISTERS_START_ADDRESS: u16 = 0x4000;
const APU_AND_IO_REGISTERS_END_ADDRESS: u16 = 0x4017;

const APU_AND_IO_CPU_TEST_MODE_REGISTERS_START_ADDRESS: u16 = 0x4018;
const APU_AND_IO_CPU_TEST_MODE_REGISTERS_END_ADDRESS: u16 = 0x401F;

const MAPPER_CONTROLLED_REGION_START_ADDRESS: u16 = 0x4020;
const MAPPER_CONTROLLED_REGION_END_ADDRESS: u16 = 0xFFFF;

pub(crate) enum Mapper {
    Nrom {
        has_32_kibibytes_prg_rom_capacity: bool
    },
}

pub(crate) trait Rom {
    fn mapper(&self) -> Mapper;
    fn read_prg(&self, index: usize) -> u8;
}

pub(crate) struct Bus {
    cpu_ram: [u8; 2 * BYTES_ON_A_KIBIBYTE],
    rom: Box<dyn Rom>,
}

#[derive(Error, Debug)]
enum BusError {
    #[error("Unable to read from the shared memory address space: {0}")]
    CannotRead(&'static str),

    #[error("Unable to write to the shared memory address space: {0}")]
    CannotWrite(&'static str),
}

impl Bus {
    pub(crate) fn new<T: Rom + 'static>(rom: T) -> Bus {
        Bus {
            cpu_ram: [0; 2 * BYTES_ON_A_KIBIBYTE],
            rom: Box::new(rom),
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

            MAPPER_CONTROLLED_REGION_START_ADDRESS..=MAPPER_CONTROLLED_REGION_END_ADDRESS => self.read_mapper_controlled_region(address),
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

            MAPPER_CONTROLLED_REGION_START_ADDRESS..=MAPPER_CONTROLLED_REGION_END_ADDRESS => self.write_mapper_controlled_region(address, value),
        }
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test to do:
    // - CPU RAM read/write
    // - CPU RAM MIRRORS read/write (all of them)
    // - Mappers read/write:
    //      - NROM
    // Probably more...
}