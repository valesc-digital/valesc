//! Holds the implementation of the instructions related with branching..

use super::CpuStatusFlags;
use crate::{cpu::{Cpu, CpuError, InstructionData}, U16Ex};

impl Cpu {
    /// Branch if the carry flag is set.
    fn branch_if_carry_set(&mut self, arg_1: u8) -> Result<InstructionData, CpuError> {
        let mut idle_cycles = 1;

        let increase_program_counter = if self.status.contains(CpuStatusFlags::Carry) {
            arg_1 as u16
        } else {
            2
        };

        if (self.program_counter + increase_program_counter).get_upper_byte() != self.program_counter.get_upper_byte() {
            idle_cycles += 2;
        }

        Ok(InstructionData {
            idle_cycles,
            assembly: format!(""),
            increase_program_counter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;
}
