//! Holds the implementation of the modified 2A03 CPU used by the NES.

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::Add;

use bitflags::bitflags;
use log::{error, trace};
use thiserror::Error;

use crate::bus::{Bus, BusError};
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

pub enum AddressingMode {
    Absolute,
}

pub enum Instruction {
    Jump(AddressingMode)
}

pub struct CpuStep {

}

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

    /// The bus and board of the NES system.
    bus: Bus,

    remaining_steps: VecDeque<CpuStep>,
}

#[derive(Error, Debug)]
/// Errors that may happen when interacting with the CPU.
pub enum CpuError {
    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),
}

impl Cpu {
    /// Creates a new [Cpu].
    pub fn new(cartridge: Box<dyn Cartridge>) -> Cpu {
        Cpu::new_with_program_counter(cartridge, 0x8000)
    }

    /// Creates a new [Cpu] given a custom program counter (PC) value.
    pub fn new_with_program_counter(cartridge: Box<dyn Cartridge>, program_counter: u16) -> Cpu {
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,

            status: CpuStatusFlags::Decimal | CpuStatusFlags::B,
            stack_pointer: 0xFD,
            program_counter,

            bus: Bus::new(cartridge),
            remaining_steps: VecDeque::new(),
        }
    }

    /// Do a cycle on the CPU.
    pub fn cycle() {

    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
