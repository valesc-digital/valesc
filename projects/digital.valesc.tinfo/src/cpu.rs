//! Holds the implementation of the modified 6502 CPU used by the NES.

use std::fmt::format;
use std::ops::Add;

use bitflags::bitflags;
use log::trace;
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

/// The 6502 based CPU used by the NES.
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

    /// The current cycle number of the CPU.
    cycle: u64,

    /// The number of cycles the CPU should skip processing instructions for timing.
    idle_cycles: u8,
}

/// The default program counter (PC) of the CPU.
const DEFAULT_PROGRAM_COUNTER: u16 = 0x8000;

#[derive(Error, Debug)]
/// Errors that may happen when interacting with the CPU.
pub enum CpuError {
    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),

    #[error("The selected addressing mode is not valid for the executed instruction")]
    InvalidAddressingMode,
}

enum AddressingMode {
    Absolute,
    Immediate,
}

/// Data returned after processing an instruction.
pub struct InstructionData {
    /// The number of cycles the CPU should skip for correct timing.
    pub idle_cycles: u8,

    /// Formatted string of the processed instruction as it should be written in assembly.
    pub assembly: String,
}

/// Data returned after processing a CPU step.
pub struct StepData {
    /// Information log entry related to the step formatted as the [Nestopia emulator](http://0ldsk00l.ca/nestopia/),
    /// useful for testing with `nestest.nes` and `nestest.log`.
    pub nestopia_log: String,

    /// The data related to the processed instruction.
    pub instruction_data: InstructionData,
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
            cycle: 0,
            idle_cycles: 0,
        }
    }

    /// Step once the CPU.
    pub fn step(&mut self) -> Result<Option<StepData>, CpuError> {
        self.cycle += 1;

        if self.idle_cycles > 0 {
            self.idle_cycles -= 1;
            trace!("Idle cycle, skipping");

            return Ok(None);
        }

        let opcode = self
            .bus
            .read(self.program_counter)
            .map_err(CpuError::BusError)?;

        let arg_1 = self
            .bus
            .read(self.program_counter + 1)
            .map_err(CpuError::BusError)?;

        let arg_2 = self
            .bus
            .read(self.program_counter + 2)
            .map_err(CpuError::BusError)?;

        let old_program_counter = self.program_counter;

        let instruction_data: InstructionData = match opcode {
            0x4C => self.jmp(arg_1, arg_2, AddressingMode::Absolute),
            0xA2 => self.ldx(arg_1, AddressingMode::Immediate),
            _ => unimplemented!(),
        }?;

        self.idle_cycles = instruction_data.idle_cycles;

        let log_column_one = format!(
            "{old_program_counter:X}  {opcode:02X} {arg_1:02X} {arg_2:02X}  {}",
            instruction_data.assembly
        );
        let log_padding = " ".repeat(32 - instruction_data.assembly.len());
        let log_column_two = format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02} SP:{:02X}",
            self.accumulator,
            self.register_x,
            self.register_y,
            self.status.bits(),
            self.stack_pointer
        );

        Ok(Some(StepData {
            nestopia_log: format!(
                "{log_column_one}{log_padding}{log_column_two}"
            ),
            instruction_data,
        }))
    }

    /// Set the program counter to the desired value.
    fn jmp(
        &mut self,
        arg_1: u8,
        arg_2: u8,
        addressing_mode: AddressingMode,
    ) -> Result<InstructionData, CpuError> {
        match addressing_mode {
            AddressingMode::Absolute => {
                self.program_counter = (arg_1 as u16) | ((arg_2 as u16) << 8);

                Ok(InstructionData {
                    idle_cycles: 2,
                    assembly: format!("JMP ${:04X}", self.program_counter),
                })
            }

            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    /// Load a value to the register X.
    fn ldx(
        &mut self,
        arg_1: u8,
        addressing_mode: AddressingMode,
    ) -> Result<InstructionData, CpuError> {
        match addressing_mode {
            AddressingMode::Immediate => {
                self.register_x = arg_1;

                Ok(InstructionData {
                    idle_cycles: 1,
                    assembly: format!("LDX #${arg_1:02X}"),
                })
            }

            _ => Err(CpuError::InvalidAddressingMode),
        }
    }

    /// Store a value from the register X.
    fn stx(
        &mut self,
        arg_1: u8,
        arg_2: u8,
        addressing_mode: AddressingMode,
    ) -> Result<InstructionData, CpuError> {
        match addressing_mode {
            AddressingMode::Immediate => {
                self.register_x = arg_1;

                Ok(InstructionData {
                    idle_cycles: 1,
                    assembly: format!("LDX #${arg_1:02X}"),
                })
            }

            _ => Err(CpuError::InvalidAddressingMode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_PROGRAM_COUNTER: usize = 0x8000;

    struct MockCartridge {
      prg_data: Vec<u8>,
    }

    impl MockCartridge {
        fn new(prg_data: Vec<u8>) -> MockCartridge {
            MockCartridge {
                prg_data
            }
        }
    }

    impl Cartridge for MockCartridge {
        unsafe fn read(&self, address: u16) -> Result<u8, crate::cartridge::CartridgeError> {
            Ok(self.prg_data[address as usize - DEFAULT_PROGRAM_COUNTER])
        }

        unsafe fn write(&mut self, _address: u16, _value: u8) -> Result<(), crate::cartridge::CartridgeError> {
            Ok(())
        }
    }

    #[test]
    fn test_jmp_absolute() {
        let cartridge = MockCartridge::new(vec![
            // JMP $5533
            0x4C,
            0x33,
            0x55,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        
        let instruction_data = cpu.step().unwrap().unwrap().instruction_data;

        assert_eq!(instruction_data.assembly, "JMP $5533");
        assert_eq!(instruction_data.idle_cycles, 2);
        assert_eq!(cpu.program_counter, 0x5533);
    }

    // MISSING: Be able to write to the RAM
    /*
    #[test]
    fn test_jmp_indirect() {
        let cartridge = MockCartridge::new(vec![
            // JMP ($8011) = DB7E
            0x6C,
            0x11,
            0x80,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        
        let instruction_data = cpu.step().unwrap().unwrap().instruction_data;

        assert_eq!(instruction_data.assembly, "JMP ($5533) = DB7E");
        assert_eq!(instruction_data.idle_cycles, 4);
        assert_eq!(cpu.program_counter, 0x5533);
    }
    */

    #[test]
    fn test_ldx_immediate() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$CC
            0xA2,
            0xCC,

            // Dummy value because the implementation needs at least 3 PRG ROM bytes to work
            // NOP
            0xEA
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        
        let instruction_data = cpu.step().unwrap().unwrap().instruction_data;

        assert_eq!(instruction_data.assembly, "LDX #$CC");
        assert_eq!(instruction_data.idle_cycles, 1);
        assert_eq!(cpu.register_x, 0xCC);
    }

    #[test]
    fn test_stx_absolute() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$CC
            0xA2,
            0xCC,

            // STX
            0x8E,
            0x00,
            0x00,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        
        let instruction_data = cpu.step().unwrap().unwrap().instruction_data;

        assert_eq!(instruction_data.assembly, "STX $0000 = CC");
        assert_eq!(instruction_data.idle_cycles, 3);
        assert_eq!(cpu.bus.read(0x0000).unwrap(), 0xCC);
    }
}