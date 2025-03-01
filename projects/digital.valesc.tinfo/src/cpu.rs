//! Holds the implementation of the modified 2A03 CPU used by the NES.

use core::panic;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::io::Read;
use std::ops::Add;
use std::rc::Weak;
use std::time::Instant;

use bitflags::bitflags;
use log::{error, trace};
use thiserror::Error;

use crate::bus::{self, Bus, BusError};
use crate::cartridge::{self, Cartridge};
use crate::{build_address, rom};

bitflags! {
    #[derive(Debug)]
    /// Attributes can be applied to the CPU status/flags register.
    pub struct CpuStatusFlags: u8 {
        /// Carry a bit remaining by some instructions.
        const Carry = 1 << 0;

        /// Some instructions set it if the result value is zero.
        const Zero = 1 << 1;

        /// If the interrupts of the CPU, expect for the Non Maskable Interrupt (NMI), are disabled.
        const InterruptsDisabled = 1 << 2 ;

        /// If the CPU is in decimal mode, junk value as it is not available on the original NES.
        const Decimal = 1 << 3;

        /// Informs if the last interrupt was triggered by a NMI/IRQ (zero value) or a BRK/PHP (one value).
        /// See also: [The B flag info in the NESDev wiki](https://www.nesdev.org/wiki/Status_flags#The_B_flag)
        const B = 1 << 4;

        /// Stub value, does nothing. It's only defined to be able to set it to a value and match
        /// other emulators settings.
        const Stub = 1 << 5; // Does noting, always 0b1

        /// Some instructions set it if the result value overflow.
        const Overflow = 1 << 6;

        /// Some instructions set it if the result value is negative.
        const Negative = 1 << 7;
    }
}

/// The address to the first byte of the stack in the bus memory space.
const STACK_ADDRESS: u16 = 0x0100;

/// The 2A03 CPU used by the NES.
pub struct Cpu {
    /// Accumulator register, also know as register `A`, used by some ALU operations.
    accumulator: u8,

    /// Generic index register X.
    register_x: u8,

    /// Generic index register Y.
    register_y: u8,

    /// Status register, holds different bit flags that reports the state of the CPU.
    status: CpuStatusFlags,

    /// Current offset from the start of the stack address.
    stack_pointer: u8,

    /// The address of the next instruction to process.
    program_counter: u16,

    current_instruction: Instruction,
    current_instruction_cycle: u8,

    bus: Bus,

    /// The 2A05 CPU can access data retrived from previous cycles of the same instruction,
    /// cycles can store here well-known internal data.
    cache: Vec<u8>,
}

#[derive(Error, Debug)]
/// Errors that may happen when interacting with the CPU.
pub enum CpuError {
    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),

    #[error("Running the cycle failed: {0}")]
    /// Accessing the bus failed
    InstructionError(#[from] CycleError),
}

#[derive(Debug)]
// To much of a hassle to document all of them
#[allow(clippy::missing_docs_in_private_items)]
/// The different instructions that the CPU can run.
enum Instruction {
    Stub,
    JumpAbsolute,
}

#[derive(Error, Debug)]
/// Errors that can happen when running a cycle.
pub enum CycleError {
    #[error("The requested instruction step is out of bounds")]
    /// The requested instruction step is out of bounds
    InstructionStepOutOfBounds,

    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),
}

impl Cpu {
    /// Create a new [Cpu].
    pub fn new(cartridge: Box<dyn Cartridge>) -> Cpu {
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,

            status: CpuStatusFlags::Decimal | CpuStatusFlags::B,
            stack_pointer: 0xFD,
            program_counter: 0xC000,

            current_instruction: Instruction::Stub,
            current_instruction_cycle: 1,

            bus: Bus::new(cartridge),
            cache: vec![],
        }
    }

    /// Run a cycle of the CPU.
    pub fn cycle(&mut self) -> Result<(), CpuError> {
        if self.current_instruction_cycle == 1 {
            self.current_instruction = Self::dispatch_opcode(self.bus.read(self.program_counter)?);
            self.program_counter += 1;
            self.current_instruction_cycle += 1;

            return Ok(());
        }

        let instruction_ended = match self.current_instruction {
            Instruction::JumpAbsolute => self.jump_absolute(),
            Instruction::Stub => panic!("The stub instruction should never go beyond step 1!"),
        }?;

        self.current_instruction_cycle += 1;

        if instruction_ended {
            // This will retrigger the opcode dispatch cycle
            self.current_instruction_cycle = 1;
            self.cache.clear();
        }

        Ok(())
    }

    fn dispatch_opcode(opcode: u8) -> Instruction {
        match opcode {
            0x4C => Instruction::JumpAbsolute,
            _ => unimplemented!("The opcode {opcode:02X} is not implemented yet!")
        }
    }
    
    fn jump_absolute(&mut self) -> Result<bool, CycleError> {
        match self.current_instruction_cycle {
            2 => {
                self.cache.push(self.bus.read(self.program_counter)?);
                self.program_counter += 1;

                Ok(false)
            },

            3 => {
                let program_counter_adress_high_byte = self.bus.read(self.program_counter)?;
                self.program_counter = build_address(self.cache[0], program_counter_adress_high_byte);

                Ok(true)
            }

            _ => Err(CycleError::InstructionStepOutOfBounds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
