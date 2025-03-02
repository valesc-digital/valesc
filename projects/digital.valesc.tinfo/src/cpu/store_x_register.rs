//! Holds the implementation of the `STX` instruction.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;


impl Cpu {
    /// Implements the zero page store X register instruction data.
    pub(super) fn store_x_register_zero_page_instruction(&mut self) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: None,
            assembly: format!("STX #${arg_1:02X} = {:02X}", self.bus.read(build_address(arg_1, 0x00))?),
            idle_cycles: 2,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the zero page store X register instruction cycles.
    cpu, store_x_register_zero_page_cycles,

    2, false => {
        cpu.cache.push(cpu.read_program_counter()?);
        cpu.program_counter += 1;
    },

    3, true => {
        cpu.bus.write(
            build_address(cpu.cache[0], 0x00),
        cpu.register_x)?;
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_stx_zero_page() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$5C
            0xA2, 0x5C,

            // STX $EE
            0x86, 0xEE,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));
        cpu.bus.write(0x00EE, 0xAB).unwrap();

        cpu.run_full_instruction();

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "STX #$EE = AB");
        assert_eq!(instruction_data.idle_cycles, 2);

        assert_eq!(cpu.program_counter, 0x8003);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8004);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8004);
        assert_eq!(cpu.bus.read(0x00EE).unwrap(), 0x5C);
    }
}