//! Holds the implementation of the instructions related with the carry flag.

use super::CpuStatusFlags;
use crate::cpu::{Cpu, CpuError, InstructionData};

impl Cpu {
    /// Set the carry flag of the CPU.
    pub(super) fn set_carry_flag(&mut self) -> Result<InstructionData, CpuError> {
        self.status |= CpuStatusFlags::Carry;

        Ok(InstructionData {
            idle_cycles: 1,
            assembly: String::from("SEC"),
            increase_program_counter: 1,
        })
    }

    /// Clear the carry flag of the CPU.
    pub(super) fn clear_carry_flag(&mut self) -> Result<InstructionData, CpuError> {
        self.status -= CpuStatusFlags::Carry;

        Ok(InstructionData {
            idle_cycles: 1,
            assembly: String::from("CLC"),
            increase_program_counter: 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_sec() {
        let cartridge = MockCartridge::new(vec![
            // SEC
            0x38, NOP, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.status -= CpuStatusFlags::Carry;

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "SEC");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert!(cpu.status.contains(CpuStatusFlags::Carry));
    }

    #[test]
    fn test_clc() {
        let cartridge = MockCartridge::new(vec![
            // CLC
            0x18, NOP, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.status |= CpuStatusFlags::Carry;

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "CLC");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert!(!cpu.status.contains(CpuStatusFlags::Carry));
    }
}
