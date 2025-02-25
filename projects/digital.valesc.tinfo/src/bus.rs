//! Holds the implementation of a memory bus for the NES.

use thiserror::Error;
use rand::prelude::*;

use crate::{cartridge::{Cartridge, CartridgeError}, BYTES_ON_A_KIBIBYTE};

/// The address of the first byte of the CPU RAM.
const CPU_RAM_WITH_MIRRORING_START_ADDRESS: u16 = 0x0000;

/// The address of the last byte of the CPU RAM after its three mirrors.
const CPU_RAM_WITH_MIRRORING_END_ADDRESS: u16 = 0x1FFF;

/// The address of the first byte of the PPU registers.
const PPU_REGISTERS_WITH_MIRRORING_START_ADDRESS: u16 = 0x2000;

/// The address of the last byte of the PPU registers after all the mirrors.
const PPU_REGISTERS_WITH_MIRRORING_END_ADDRESS: u16 = 0x3FFF;

/// The address of the first byte of the APU and IO registers.
const APU_AND_IO_REGISTERS_START_ADDRESS: u16 = 0x4000;

/// The address of the last byte of the APU and IO registers.
const APU_AND_IO_REGISTERS_END_ADDRESS: u16 = 0x4017;

/// The address of the first byte of the APU and IO registers available only on the CPU Test Mode.
const APU_AND_IO_CPU_TEST_MODE_REGISTERS_START_ADDRESS: u16 = 0x4018;

/// The address of the last byte of the APU and IO registers available only on the CPU Test Mode.
const APU_AND_IO_CPU_TEST_MODE_REGISTERS_END_ADDRESS: u16 = 0x401F;

/// The address of the first byte of the cartridge mapper chip controlled address range.
const CARTRIDGE_CONTROLLED_REGION_START_ADDRESS: u16 = 0x4020;

/// The address of the last byte of the cartridge mapper chip controlled address range.
const CARTRIDGE_CONTROLLED_REGION_END_ADDRESS: u16 = 0xFFFF;

/// Emulation of the chips and boards related to memory address management. 
pub(crate) struct Bus {
    /// The RAM of the CPU.
    cpu_ram: [u8; 2 * BYTES_ON_A_KIBIBYTE],

    /// The inserted cartridge in the board.
    pub cartridge: Box<dyn Cartridge>,
}

#[derive(Error, Debug)]
/// Errors that may happens when interacting with the bus.
pub(crate) enum BusError {
    #[error("Unable to read from the shared memory address space: {0}")]
    /// Unable to read from the shared memory address space.
    CannotRead(&'static str),

    #[error("Unable to write to the shared memory address space: {0}")]
    /// Unable to write to the shared memory address space.
    CannotWrite(&'static str),

    #[error("Unable to access to the cartridge: {0}")]
    /// Unable to access to the cartridge.
    CartridgeError(#[from] CartridgeError),
}

impl Bus {
    /// Create a new [Bus].
    pub(crate) fn new(cartridge: Box<dyn Cartridge>) -> Bus {
        // The CPU RAM should be randomized to emulate the undefined state of the bits on startup,
        // used on some games as a pseudo RNG

        let mut rng = rand::rng();
        let cpu_ram: Vec<u8> = rng.random_iter().take(2 * BYTES_ON_A_KIBIBYTE).collect();

        Bus {
            cpu_ram: cpu_ram.try_into().unwrap(),
            cartridge,
        }
    }

    /// Request a read to the bus. 
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

            CARTRIDGE_CONTROLLED_REGION_START_ADDRESS..=CARTRIDGE_CONTROLLED_REGION_END_ADDRESS => unsafe { self.cartridge.read(address).map_err(BusError::CartridgeError) },
        }
    }

    /// Write a byte to a memory address in the bus.
    pub(crate) fn write(&mut self, address: u16, value: u8) -> Result<(), BusError> {
        println!("Wrote @ {address:#02X}: {value:#02X}");
        
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

            CARTRIDGE_CONTROLLED_REGION_START_ADDRESS..=CARTRIDGE_CONTROLLED_REGION_END_ADDRESS => unsafe { self.cartridge.write(address, value).map_err(BusError::CartridgeError) },
        }
    }
}
