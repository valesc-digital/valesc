//! Holds the implementation of the `NOP` instruction.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;


impl Cpu {
    /// Implements the implied no operation instruction data.
    pub(super) fn no_operation_implied_instruction(&mut self) -> Result<InstructionData, BusError> {
        Ok(InstructionData {
            arg_1: None,
            arg_2: None,
            assembly: String::from("NOP"),
            idle_cycles: 1,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the implied no operation instruction cycles.
    cpu, no_operation_cycles,

    2, true => {
        // Dummy read
        let _ = cpu.read_program_counter();
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_nop_immediate() {
        let cartridge = MockCartridge::new(vec![
            // NOP
            0xEA,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "NOP");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
    }
}