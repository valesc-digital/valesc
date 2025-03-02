//! Holds the implementation of the `JSR` instruction.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::U16Ex;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;

use super::STACK_ADDRESS;

impl Cpu {
    /// Implements the absolute jump instruction data.
    pub(super) fn jump_to_subroutine_absolute_instruction(&mut self) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;
        let arg_2 = self.bus.read(self.program_counter + 2)?;
        
        let address = build_address(
            arg_1,
            arg_2
        );

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: Some(arg_2),
            assembly: format!("JSR ${address:02X}"),
            idle_cycles: 5,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the absolute jump to subroutine instruction cycles.
    cpu, jump_to_subroutine_absolute_cycles,

    2, false => {
        cpu.cache.push(cpu.read_program_counter()?);
        cpu.program_counter += 1;
    },

    3, false => {
        // Internal operation
        let _ = cpu.bus.read(0x100)?;
    },

    4, false => {
        cpu.stack_push(cpu.program_counter.upper_byte())?;
    },

    5, false => {
        cpu.stack_push(cpu.program_counter.lower_byte())?;
    },

    6, true => {
        let program_counter_high = cpu.read_program_counter()?;

        cpu.program_counter = build_address(cpu.cache[0], program_counter_high);
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_jsr_immediate() {
        let cartridge = MockCartridge::new(vec![
            // JSR $77EE
            0x20, 0xEE, 0x77
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "JSR $77EE");
        assert_eq!(instruction_data.idle_cycles, 5);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
        assert_eq!(cpu.bus.read(0x01FD).unwrap(), 0x80);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
        assert_eq!(cpu.bus.read(0x01FC).unwrap(), 0x02);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x77EE);
    }
}