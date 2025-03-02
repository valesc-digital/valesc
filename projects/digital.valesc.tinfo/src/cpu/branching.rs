//! Implements the instructions related to branching the code flow in CPU.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::U16Ex;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;

use super::CpuStatusFlags;

impl Cpu {
    /// Implements a generic implied branching instruction data.
    pub(super) fn branch_instruction(&mut self, status_flag: CpuStatusFlags, not: bool) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;
        
        let new_program_counter = self.program_counter + 2 + arg_1 as u16;

        let mut idle_cycles = 1;

        let contains_status_flag = self.status.contains(status_flag);
        if (contains_status_flag && !not) || (!contains_status_flag && not) {
            idle_cycles += 1;

            if self.program_counter.upper_byte() != new_program_counter.upper_byte() {
                idle_cycles += 1;
            }
        }

        let prefix = match not {
            false => {
                match status_flag {
                    CpuStatusFlags::Carry => "BCS",
                    CpuStatusFlags::Zero => "BEQ",
                    CpuStatusFlags::Overflow => "BVS",
                    CpuStatusFlags::Negative => "BMI",
                    _ => unimplemented!(),
                }
            },

            true => {
                match status_flag {
                    CpuStatusFlags::Carry => "BCC",
                    CpuStatusFlags::Zero => "BNE",
                    CpuStatusFlags::Overflow => "BVC",
                    CpuStatusFlags::Negative => "BPL",
                    _ => unimplemented!(),
                }
            }
        };

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: None,
            assembly: format!("{prefix} ${new_program_counter:04X}"),
            idle_cycles,
        })
    }

    /// Implements the implied set carry flag instruction cycles.
    pub(super) fn branch_cycles(&mut self, status_flag: CpuStatusFlags, not: bool) -> Result<bool, CycleError> {
        match self.current_instruction_cycle {
            2 => {
                let offset = self.read_program_counter()?;
                self.program_counter += 1;

                let contains_status_flag = self.status.contains(status_flag);
                if !((contains_status_flag && !not) || (!contains_status_flag && not)) {
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

    fn branching_relative_no_branching(opcode: u8, assembly_text: &str, not: bool, status_flag: CpuStatusFlags) {
        let cartridge = MockCartridge::new(vec![
            opcode,
            0x20,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        if not {
            cpu.status |= status_flag;
        }

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, format!("{assembly_text} $8022"));
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_branching_relative_no_branching_bcs() {
        branching_relative_no_branching(0xB0, "BCS", false, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_no_branching_bcc() {
        branching_relative_no_branching(0x90, "BCC", true, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_no_branching_beq() {
        branching_relative_no_branching(0xF0, "BEQ", false, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_no_branching_bne() {
        branching_relative_no_branching(0xD0, "BNE", true, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_no_branching_bvs() {
        branching_relative_no_branching(0x70, "BVS", false, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_no_branching_bvc() {
        branching_relative_no_branching(0x50, "BVC", true, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_no_branching_bmi() {
        branching_relative_no_branching(0x30, "BMI", false, CpuStatusFlags::Negative);
    }

    #[test]
    fn test_branching_relative_no_branching_bpl() {
        branching_relative_no_branching(0x10, "BPL", true, CpuStatusFlags::Negative);
    }

    fn branching_relative_branching_same_page(opcode: u8, assembly_text: &str, not: bool, status_flag: CpuStatusFlags) {
        let cartridge = MockCartridge::new(vec![
            opcode,
            0x20,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        if !not {
            cpu.status |= status_flag;
        }

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, format!("{assembly_text} $8022"));
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8022);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bcs() {
        branching_relative_branching_same_page(0xB0, "BCS", false, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bcc() {
        branching_relative_branching_same_page(0x90, "BCC", true, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_branching_same_page_beq() {
        branching_relative_branching_same_page(0xF0, "BEQ", false, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bne() {
        branching_relative_branching_same_page(0xD0, "BNE", true, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bvs() {
        branching_relative_branching_same_page(0x70, "BVS", false, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bvc() {
        branching_relative_branching_same_page(0x50, "BVC", true, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bmi() {
        branching_relative_branching_same_page(0x30, "BMI", false, CpuStatusFlags::Negative);
    }

    #[test]
    fn test_branching_relative_branching_same_page_bpl() {
        branching_relative_branching_same_page(0x10, "BPL", true, CpuStatusFlags::Negative);
    }

    fn branching_relative_branching_page_change(opcode: u8, assembly_text: &str, not: bool, status_flag: CpuStatusFlags) {
        let cartridge = MockCartridge::new(vec![
            opcode,
            0xFE,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        if !not {
            cpu.status |= status_flag;
        }

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, format!("{assembly_text} $8100"));
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

    #[test]
    fn test_branching_relative_branching_page_change_bcs() {
        branching_relative_branching_page_change(0xB0, "BCS", false, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bcc() {
        branching_relative_branching_page_change(0x90, "BCC", true, CpuStatusFlags::Carry);
    }

    #[test]
    fn test_branching_relative_branching_page_change_beq() {
        branching_relative_branching_page_change(0xF0, "BEQ", false, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bne() {
        branching_relative_branching_page_change(0xD0, "BNE", true, CpuStatusFlags::Zero);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bvs() {
        branching_relative_branching_page_change(0x70, "BVS", false, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bvc() {
        branching_relative_branching_page_change(0x50, "BVC", true, CpuStatusFlags::Overflow);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bmi() {
        branching_relative_branching_page_change(0x30, "BMI", false, CpuStatusFlags::Negative);
    }

    #[test]
    fn test_branching_relative_branching_page_change_bpl() {
        branching_relative_branching_page_change(0x10, "BPL", true, CpuStatusFlags::Negative);
    }
}