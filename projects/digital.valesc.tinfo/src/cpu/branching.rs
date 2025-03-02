//! Implements the instructions related to branching the code flow in CPU.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::U16Ex;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;

use super::CpuStatusFlags;

impl Cpu {
    /// Implements the implied set carry flag instruction data.
    pub(super) fn branch_if_carry_set_relative_instruction(&mut self) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;
        
        let new_program_counter = self.program_counter + 2 + arg_1 as u16;

        let mut idle_cycles = 1;
        if self.status.contains(CpuStatusFlags::Carry) {
            idle_cycles += 1;

            if self.program_counter.upper_byte() != new_program_counter.upper_byte() {
                idle_cycles += 1;
            }
        }

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: None,
            assembly: format!("BCS ${new_program_counter:04X}"),
            idle_cycles,
        })
    }

    /// Implements the implied set carry flag instruction cycles.
    pub(super) fn branch_if_carry_set_relative_cycles(&mut self) -> Result<bool, CycleError> {
        match self.current_instruction_cycle {
            2 => {
                let offset = self.read_program_counter()?;
                self.program_counter += 1;

                if !self.status.contains(CpuStatusFlags::Carry) {
                    return Ok(true);
                }

                self.cache.push(offset);

                Ok(false)
            },

            3 => {
                let _ = self.bus.read(self.program_counter + 1);
                let new_program_counter = self.program_counter + self.cache[0] as u16;

                if new_program_counter.upper_byte() == self.program_counter.upper_byte() {
                    self.program_counter = new_program_counter;
                    return Ok(true)
                }

                // Force broken PC
                self.program_counter = build_address(
                    new_program_counter.lower_byte(),
                    self.program_counter.upper_byte()
                );

                Ok(false)
            }

            4 => {
                let _ = self.read_program_counter();
                // Fix PCH.
                self.program_counter = build_address(
                    self.program_counter.lower_byte(),
                    self.program_counter.upper_byte() + 1
                );

                Ok(true)
            }

            _ => Err(CycleError::InstructionCycleOutOfBounds),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_branch_if_carry_set_relative_no_branching() {
        let cartridge = MockCartridge::new(vec![
            // BCS $20
            0xB0,
            0x20,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "BCS $8022");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_branch_if_carry_set_relative_branching_same_page() {
        let cartridge = MockCartridge::new(vec![
            // BCS $20
            0xB0,
            0x20,

            // Dummy value
            0xFF,
            0xFF,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.status |= CpuStatusFlags::Carry;

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "BCS $8022");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8022);
    }

    #[test]
    fn test_branch_if_carry_set_relative_branching_page_change() {
        let cartridge = MockCartridge::new(vec![
            // BCS $FE
            0xB0,
            0xFE,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.status |= CpuStatusFlags::Carry;

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "BCS $8100");
        assert_eq!(instruction_data.idle_cycles, 3);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        // Check if the incorrect value is being saved in propose
        assert_eq!(cpu.program_counter, 0x8000);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8100);
    }
}