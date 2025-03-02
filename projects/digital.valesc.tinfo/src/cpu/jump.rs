//! Holds the implementation of the `JMP` instruction.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;

impl Cpu {
    /// Implements the absolute jump instruction data.
    pub(super) fn jump_absolute_instruction(&mut self) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;
        let arg_2 = self.bus.read(self.program_counter + 2)?;
        
        let address = build_address(
            arg_1,
            arg_2
        );

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: Some(arg_2),
            assembly: format!("JMP ${address:02X}"),
            idle_cycles: 2,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the absolute jump instruction cycles.
    cpu, jump_absolute_cycles,

    2, false => {
        cpu.cache.push(cpu.read_program_counter()?);
        cpu.program_counter += 1;
    },

    3, true => {
        let program_counter_address_upper_byte = cpu.read_program_counter()?;
        cpu.program_counter =
            build_address(
                cpu.cache[0],
                program_counter_address_upper_byte
            );
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_jmp_absolute() {
        let cartridge = MockCartridge::new(vec![
            // JMP $5533
            0x4C, 0x33, 0x55
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "JMP $5533");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x5533);
    }
}