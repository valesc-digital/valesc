//! Holds the implementation of the modified 6502 CPU used by the NES.

mod carry_flag;
mod jump;
mod load_x_register;
mod store_x_register;
mod subroutine;
mod branching;

use std::cmp::Ordering;

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

const STACK_ADDRESS: u16 = 0x0100;

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

#[derive(Error, Debug)]
/// Errors that may happen when interacting with the CPU.
pub enum CpuError {
    #[error("Accessing the bus failed: {0}")]
    /// Accessing the bus failed
    BusError(#[from] BusError),
}

/// Data returned after processing an instruction.
pub struct InstructionData {
    /// The number of cycles the CPU should skip for correct timing.
    pub idle_cycles: u8,

    /// Formatted string of the processed instruction as it should be written in assembly.
    pub assembly: String,

    /// The value the program counter should be increased.
    pub increase_program_counter: u16,
}

impl InstructionData {
    /// An instruction that does nothing.
    fn stub_instruction() -> InstructionData {
        InstructionData {
            idle_cycles: 1,
            assembly: String::from("NOP"),
            increase_program_counter: 1,
        }
    }
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

        let log_column_two = format!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02} SP:{:02X}",
            self.accumulator,
            self.register_x,
            self.register_y,
            self.status.bits(),
            self.stack_pointer
        );

        let instruction_data = match opcode {
            0x4C => self.jump_absolute(arg_1, arg_2),
            0x6C => self.jump_indirect(arg_1, arg_2),

            0xA2 => self.load_x_register_immediate(arg_1),
            0x8E => self.store_x_register_absolute(arg_1, arg_2),
            0x86 => self.store_x_register_zero_page(arg_1),

            0x20 => self.jump_to_subroutine(arg_1, arg_2),

            0x38 => self.set_carry_flag(),
            0x18 => self.clear_carry_flag(),

            0xEA => Ok(InstructionData::stub_instruction()),

            opcode => {
                error!("UNKNOWN INSTRUCTION: 0x{opcode:02X}");
                unimplemented!()
            }
        }?;

        self.idle_cycles = instruction_data.idle_cycles;
        self.program_counter += instruction_data.increase_program_counter;

        // Nestopia logging format
        let log_column_one = format!(
            "{old_program_counter:X}  {opcode:02X} {arg_1:02X} {arg_2:02X}  {}",
            instruction_data.assembly
        );
        let log_padding = " ".repeat(32 - instruction_data.assembly.len());

        Ok(Some(StepData {
            nestopia_log: format!("{log_column_one}{log_padding}{log_column_two}"),
            instruction_data,
        }))
    }

    /// Given a value set the cpu flags related to the positive, negative or zero value
    /// of the given number.
    fn set_signedness(&mut self, value: u8) {
        match (value as i8).cmp(&0) {
            Ordering::Greater => {
                self.status -= CpuStatusFlags::Negative;
                self.status -= CpuStatusFlags::Zero;
            }

            Ordering::Equal => {
                self.status |= CpuStatusFlags::Zero;
                self.status -= CpuStatusFlags::Negative;
            }

            Ordering::Less => {
                self.status |= CpuStatusFlags::Negative;
                self.status -= CpuStatusFlags::Zero;
            }
        }
    }

    /// Push a value to the stack.
    fn stack_push(&mut self, value: u8) -> Result<(), BusError> {
        self.bus.write(STACK_ADDRESS + self.stack_pointer as u16, value)?;
        self.stack_pointer -= 1;

        Ok(())
    }

    /// Pull a value from the stack.
    fn stack_pull(&mut self) -> Result<u8, BusError> {
        self.stack_pointer += 1;

        self.bus.read(STACK_ADDRESS + self.stack_pointer as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const NOP: u8 = 0xEA;
    const DEFAULT_PROGRAM_COUNTER: usize = 0x8000;

    pub(crate) struct MockCartridge {
        prg_data: Vec<u8>,
    }

    impl MockCartridge {
        pub(crate) fn new(prg_data: Vec<u8>) -> MockCartridge {
            MockCartridge { prg_data }
        }
    }

    impl Cartridge for MockCartridge {
        unsafe fn read(&self, address: u16) -> Result<u8, crate::cartridge::CartridgeError> {
            Ok(self.prg_data[address as usize - DEFAULT_PROGRAM_COUNTER])
        }

        unsafe fn write(
            &mut self,
            _address: u16,
            _value: u8,
        ) -> Result<(), crate::cartridge::CartridgeError> {
            Ok(())
        }
    }

    impl Cpu {
        pub(crate) fn quick_step(&mut self) -> InstructionData {
            let instruction_data = self.step().unwrap().unwrap().instruction_data;

            for _ in 0..instruction_data.idle_cycles {
                self.step().unwrap();
            }

            instruction_data
        }

        pub(crate) fn batch_step(&mut self, num_of_steps: usize) {
            for _ in 0..num_of_steps {
                self.quick_step();
            }
        }
    }

    #[test]
    fn test_nop() {
        let cartridge = MockCartridge::new(vec![
            // CLC
            NOP, NOP, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "NOP");
        assert_eq!(instruction_data.idle_cycles, 1);
    }
}
