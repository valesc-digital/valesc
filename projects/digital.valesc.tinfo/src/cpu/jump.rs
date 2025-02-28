//! Holds the implementation of the `JMP` instruction.

use crate::build_address;
use crate::cpu::{Cpu, CpuError, InstructionData};

impl Cpu {
    /// Make an absolute jump.
    pub(super) fn jump_absolute(
        &mut self,
        arg_1: u8,
        arg_2: u8,
    ) -> Result<InstructionData, CpuError> {
        self.program_counter = build_address(arg_1, arg_2);

        Ok(InstructionData {
            idle_cycles: 2,
            assembly: format!("JMP ${:04X}", self.program_counter),
            increase_program_counter: 0,
        })
    }

    /// Make an indirect jump.
    pub(super) fn jump_indirect(
        &mut self,
        arg_1: u8,
        arg_2: u8,
    ) -> Result<InstructionData, CpuError> {
        let first_byte_address = build_address(arg_1, arg_2);
        let lower_byte = self.bus.read(first_byte_address)?;
        let upper_byte = self.bus.read(first_byte_address + 1)?;

        self.program_counter = build_address(lower_byte, upper_byte);

        Ok(InstructionData {
            idle_cycles: 4,
            assembly: format!(
                "JMP (${first_byte_address:04X}) = {:04X}",
                self.program_counter
            ),
            increase_program_counter: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_jmp_absolute() {
        let cartridge = MockCartridge::new(vec![
            // JMP $5533
            0x4C, 0x33, 0x55, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "JMP $5533");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x5533);
    }

    #[test]
    fn test_jmp_indirect() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$5C
            0xA2, 0x5C, // STX
            0x8E, 0x00, 0x00, // LDX #$FF
            0xA2, 0xFF, // STX
            0x8E, 0x01, 0x00, // JMP ($0000) = 5CFF
            0x6C, 0x00, 0x00,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        cpu.batch_step(4);

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "JMP ($0000) = FF5C");
        assert_eq!(instruction_data.idle_cycles, 4);

        assert_eq!(cpu.program_counter, 0xFF5C);
    }
}
