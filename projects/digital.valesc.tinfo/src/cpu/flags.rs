//! Implements the instructions related to settings and clearing the flags of the CPU.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;

use super::CpuStatusFlags;

impl Cpu {
    /// Implements the implied set carry flag instruction data.
    pub(super) fn set_carry_flag_implied_instruction(&mut self) -> Result<InstructionData, BusError> {
        Ok(InstructionData {
            arg_1: None,
            arg_2: None,
            assembly: String::from("SEC"),
            idle_cycles: 2,
        })
    }

    /// Implements the implied clear carry flag instruction data.
    pub(super) fn clear_carry_flag_implied_instruction(&mut self) -> Result<InstructionData, BusError> {
        Ok(InstructionData {
            arg_1: None,
            arg_2: None,
            assembly: String::from("CLC"),
            idle_cycles: 2,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the implied set carry flag instruction cycles.
    cpu, set_carry_flag_implied_cycles,

    2, true => {
        let _ = cpu.read_program_counter();
        cpu.status |= CpuStatusFlags::Carry;
    },
);

impl_instruction_cycles!(
    /// Implements the implied set carry flag instruction cycles.
    cpu, clear_carry_flag_implied_cycles,

    2, true => {
        let _ = cpu.read_program_counter();
        cpu.status -= CpuStatusFlags::Carry;
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_set_carry_flag_implied() {
        let cartridge = MockCartridge::new(vec![
            // SEC
            0x38,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "SEC");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8001);
        assert!(cpu.status.contains(CpuStatusFlags::Carry));
    }

    #[test]
    fn test_clear_carry_flag_implied() {
        let cartridge = MockCartridge::new(vec![
            // SEC
            0x18,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.status -= CpuStatusFlags::Carry;

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "CLC");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8001);
        assert!(!cpu.status.contains(CpuStatusFlags::Carry));
    }
}