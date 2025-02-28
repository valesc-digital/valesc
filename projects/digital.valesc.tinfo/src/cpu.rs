//! Holds the implementation of the modified 2A03 CPU used by the NES.

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

use crate::bus::{self, Bus, BusError, BusRequest};
use crate::cartridge::Cartridge;

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

    last_cpu_cycle: Instant,
}

#[derive(Error, Debug)]
/// Errors that may happen when interacting with the CPU.
pub enum CpuError {
    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),

    #[error("Running the instruction failed: {0}")]
    /// Accessing the bus failed
    InstructionError(#[from] StepError),
}

#[derive(Debug)]
pub enum Instruction {
    FetchOpcode,
    JumpAbsolute,
}

pub(crate) enum StepAction {
    BusRequest(BusRequest),
    EndOfInstruction,
    ChangeInstruction(u8),
    Nothing,
}

#[derive(Error, Debug)]
pub enum StepError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("The requested instruction step is out of bounds")]
    InstructionStepOutOfBounds,
}

impl Cpu {
    /// Create a new [Cpu].
    pub fn new() -> Cpu {
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,

            status: CpuStatusFlags::Decimal | CpuStatusFlags::B,
            stack_pointer: 0xFD,
            program_counter: 0xC000,

            last_cpu_cycle: Instant::now(),

            current_instruction: Instruction::FetchOpcode,
            current_instruction_cycle: 0,
        }
    }

    /// Tick the CPU.
    pub fn tick(&mut self, bus_response: Option<u8>) -> Result<Option<BusRequest>, CpuError> {
        let step_action = match self.current_instruction {
            Instruction::FetchOpcode => self.fetch_opcode(bus_response),
            Instruction::JumpAbsolute => self.jump_absolute(bus_response),
        }?;

        let bus_request = match step_action {
            StepAction::BusRequest(bus_request) => {
                self.current_instruction_cycle += 1;

                Some(bus_request)
            }
 
            StepAction::EndOfInstruction => {
                self.current_instruction_cycle = 0;
                self.current_instruction = Instruction::FetchOpcode;

                None
            },

            StepAction::ChangeInstruction(opcode) => {
                self.current_instruction = Self::dispatch_opcode(opcode);
                self.current_instruction_cycle = 0;

                None
            }

            StepAction::Nothing => {
                self.current_instruction_cycle += 1;

                None
            },
        };

        Ok(bus_request)
    }

    fn fetch_opcode(&mut self, bus_response: Option<u8>) -> Result<StepAction, StepError> {
            match self.current_instruction_cycle {
                0 => {
                    Ok(StepAction::BusRequest(BusRequest::Read { address: self.program_counter }))
                },
    
                1 => {
                    Ok(StepAction::ChangeInstruction(bus_response.unwrap()))
                },
    
                _ => Err(StepError::InstructionStepOutOfBounds)
            }
    }

    fn dispatch_opcode(opcode: u8) -> Instruction {
        match opcode {
            0x4C => Instruction::JumpAbsolute,
            _ => unimplemented!()
        }
    }
    
    fn jump_absolute(&mut self, bus_response: Option<u8>) -> Result<StepAction, StepError> {
        match self.current_instruction_cycle {
            0 => {
                self.program_counter += 1;

                Ok(StepAction::BusRequest(BusRequest::Read { address: self.program_counter }))
            },

            1 => {
                self.program_counter += 1;
                let low_address_byte = bus_response.unwrap();

                Ok(StepAction::BusRequest(BusRequest::Read { address: self.program_counter }))
            },

            2 => {
                // COPY LOW TO PCL
                let high_address_byte = bus_response.unwrap();
                // COPY HIGH TO PCH

                Ok(StepAction::EndOfInstruction)
            }

            _ => Err(StepError::InstructionStepOutOfBounds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
