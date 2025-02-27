//! Holds the implementation of the instructions related with subroutines.

use crate::{cpu::{Cpu, CpuError, InstructionData}, U16Ex};

impl Cpu {
    /// Jump to a subroutine.
    pub(super) fn jump_to_subroutine(
        &mut self,
        arg_1: u8,
        arg_2: u8,
    ) -> Result<InstructionData, CpuError> {
        // The return address is stores as the next instruction minus one.
        let return_address = self.program_counter + 3 - 1;

        self.stack_push(return_address.get_lower_byte())?;
        self.stack_push(return_address.get_upper_byte())?;

        self.jump_absolute(arg_1, arg_2)?;

        Ok(InstructionData {
            idle_cycles: 5,
            assembly: format!("JSR ${:04X}", self.program_counter),
            increase_program_counter: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_jsr() {
        let cartridge = MockCartridge::new(vec![
            // JSR $C72D
            0x20, 0x2D, 0xC7,
        ]);

        env_logger::init();

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "JSR $C72D");
        assert_eq!(instruction_data.idle_cycles, 5);

        assert_eq!(cpu.program_counter, 0xC72D);

        // Remember that the PRG ROM data is located at 0x8000 not 0x0000
        assert_eq!(cpu.bus.read(0x100 + 0xFD).unwrap(), 0x02);
        assert_eq!(cpu.bus.read(0x100 + 0xFC).unwrap(), 0x80);
    }
}
