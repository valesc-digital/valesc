//! Holds the implementation of the modified 2A03 CPU used by the NES.

mod jump;
mod load_x_register;
mod store_x_register;

use core::panic;
use std::io::Read;

use bitflags::bitflags;
use log::error;
use thiserror::Error;

use crate::build_address;
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

    /// The number of cycles the CPU has already executed.
    cpu_cycles: u16,
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
    LoadXRegister,
    StoreXRegister,
}

#[derive(Debug)]
#[allow(missing_docs)]
/// Store a snapshot of the state of the CPU.
pub struct CpuSnapshot {
    pub accumulator: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8,
    pub stack_pointer: u8,
    pub program_counter: u16,
    pub opcode: u8,
    pub instruction_data: InstructionData,
    pub cpy_cycles: u16,
}

impl CpuSnapshot {
    /// Make a new [CpuSnapshot].
    fn new(cpu: &Cpu) -> Result<CpuSnapshot, BusError> {
        Ok(CpuSnapshot {
            accumulator: cpu.accumulator,
            register_x: cpu.register_x,
            register_y: cpu.register_y,
            status: cpu.status.bits(),
            stack_pointer: cpu.stack_pointer,
            program_counter: cpu.program_counter,
            opcode: cpu.read_program_counter()?,
            instruction_data: InstructionData {
                arg_1: None,
                arg_2: None,
                idle_cycles: 0,
                assembly: String::new(),
            },
            cpy_cycles: cpu.cpu_cycles
        })
    }
}

#[derive(Debug)]
/// Data of the running instruction,.
pub struct InstructionData {
    /// The assembly code that represents the instruction.
    pub assembly: String,

    /// The number of extra cycles is instruction is going to take.
    pub idle_cycles: u8,

    /// The first "argument" given to the instruction, if it uses one.
    pub arg_1: Option<u8>,

    /// The second "argument" given to the instruction, if it uses one.
    pub arg_2: Option<u8>,
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

/// Macro to implement the cycles of an instruction. Takes the name of the variable of the CPU struct (usually `cpu`),
/// the name of the function and the different cycles to implement, with their cycle number and a bool identifing if
/// they should end the instruction.
/// 
/// # Example
/// The implementation of a `JMP` instruction with absolute jumping.
/// ```ignore
/// impl_instruction_cycles!(
///    /// Implements the absolute jump instruction cycles.
///    cpu, jump_absolute_cycle,
///
///    2, false => {
///        cpu.cache.push(cpu.read_program_counter()?);
///        cpu.program_counter += 1;
///    },
///
///    3, true => {
///        let program_counter_address_upper_byte = cpu.read_program_counter()?;
///        cpu.program_counter =
///            build_address(
///                cpu.cache[0],
///                program_counter_address_upper_byte
///            );
///    },
///); 
///```
macro_rules! impl_instruction_cycles {
    (
        $(#[$($attrss:tt)*])*
        $self_name: ident,
        $function_name: ident,
        $($cycle_num: expr, $is_finish: expr => $cycle:expr),*,
    ) => {
        impl Cpu {
            $(#[$($attrss)*])*
            pub(crate) fn $function_name(&mut self) -> Result<bool, CycleError> {
                #[allow(unused_mut)]
                let mut $self_name = self;

                match $self_name.current_instruction_cycle {
                    $(
                        $cycle_num => {
                            $cycle

                            Ok($is_finish)
                        },
                    )*
    
                    _ => Err(CycleError::InstructionStepOutOfBounds),
                }
            }
        }
    };
}

pub(crate) use impl_instruction_cycles;

impl Cpu {
    /// Create a new [Cpu].
    pub fn new(cartridge: Box<dyn Cartridge>) -> Cpu {
        Cpu::new_with_program_counter(cartridge, 0x8000)
    }

    /// Create a new [Cpu] with the program counter set to the given value.
    pub fn new_with_program_counter(cartridge: Box<dyn Cartridge>, program_counter: u16) -> Cpu {
        Self {
            accumulator: 0,
            register_x: 0,
            register_y: 0,

            status: CpuStatusFlags::Decimal | CpuStatusFlags::B,
            stack_pointer: 0xFD,
            program_counter,

            current_instruction: Instruction::Stub,
            current_instruction_cycle: 1,

            bus: Bus::new(cartridge),
            cache: vec![],

            cpu_cycles: 6,
        }
    }

    /// Run a cycle of the CPU.
    pub fn cycle(&mut self) -> Result<Option<CpuSnapshot>, CpuError> {
        self.cpu_cycles += 1;

        if self.current_instruction_cycle == 1 {
            let mut snapshot = CpuSnapshot::new(self)?;

            self.current_instruction = Self::dispatch_opcode(self.bus.read(self.program_counter)?);
            
            snapshot.instruction_data = self.dispatch_instruction()?;

            self.program_counter += 1;
            self.current_instruction_cycle += 1;

            return Ok(Some(snapshot));
        }

        let instruction_ended = match self.current_instruction {
            Instruction::JumpAbsolute => self.jump_absolute_cycle(),
            Instruction::LoadXRegister => self.load_x_register(),
            Instruction::StoreXRegister => self.store_x_register(),
            Instruction::Stub => panic!("The stub instruction should never go beyond step 1!"),
        }?;

        self.current_instruction_cycle += 1;

        if instruction_ended {
            // This will retrigger the opcode dispatch cycle
            self.current_instruction_cycle = 1;
            self.cache.clear();
        }

        Ok(None)
    }

    /// Read a byte from the bus pointed by the program counter (PC).
    fn read_program_counter(&self) -> Result<u8, BusError> {
        self.bus.read(self.program_counter)
    }

    /// Get the matching instruction of the given opcode byte.
    fn dispatch_opcode(opcode: u8) -> Instruction {
        match opcode {
            0x4C => Instruction::JumpAbsolute,
            0xA2 => Instruction::LoadXRegister,
            0x86 => Instruction::StoreXRegister,
            _ => unimplemented!("The opcode {opcode:02X} is not implemented yet!"),
        }
    }

    /// Get the matching instruction data for the current running instruction.
    fn dispatch_instruction(&mut self) -> Result<InstructionData, BusError> {
        match self.current_instruction {
            Instruction::JumpAbsolute => self.jump_absolute_instruction(),
            Instruction::LoadXRegister => self.load_x_register_immediate_instruction(),
            Instruction::StoreXRegister => self.store_x_register_immediate_instruction(),
            Instruction::Stub => Ok(InstructionData {
                arg_1: None,
                arg_2: None,
                assembly: String::from("INVALID STUB"),
                idle_cycles: 0,
            })
        }
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
        pub(crate) fn quick_cycle(&mut self) -> InstructionData {
            let instruction_data = self.cycle().unwrap().unwrap().instruction_data;

            for _ in 0..instruction_data.idle_cycles {
                self.cycle().unwrap();
            }

            instruction_data
        }

        pub(crate) fn batch_cycle(&mut self, num_of_steps: usize) {
            for _ in 0..num_of_steps {
                self.quick_cycle();
            }
        }
    }
}