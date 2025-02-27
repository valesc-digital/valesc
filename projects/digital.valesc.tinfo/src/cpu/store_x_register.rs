//! Holds the implementation of the `STX` instruction.

use crate::build_address;
use crate::cpu::{Cpu, CpuError, InstructionData};

impl Cpu {
    /// Store a value from the register X using an absolute address.
    pub(super) fn store_x_register_absolute(
        &mut self,
        arg_1: u8,
        arg_2: u8,
    ) -> Result<InstructionData, CpuError> {
        let address = build_address(arg_1, arg_2);
        self.bus.write(address, self.register_x)?;

        Ok(InstructionData {
            idle_cycles: 3,
            assembly: format!("STX ${address:04X} = {:00X}", self.register_x),
            increase_program_counter: 3,
        })
    }

    /// Store a value from the register X using an zero paged address.
    pub(super) fn store_x_register_zero_page(
        &mut self,
        arg_1: u8,
    ) -> Result<InstructionData, CpuError> {
        self.store_x_register_absolute(arg_1, 0x00)?;

        Ok(InstructionData {
            idle_cycles: 2,
            assembly: format!("STX ${arg_1:02X} = {:00X}", self.register_x),
            increase_program_counter: 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::tests::*;

    #[test]
    fn test_stx_absolute() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$CC
            0xA2, 0xCC, // STX
            0x8E, 0x00, 0x00, 0xEA,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        cpu.quick_step();

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "STX $0000 = CC");
        assert_eq!(instruction_data.idle_cycles, 3);
        assert_eq!(cpu.bus.read(0x0000).unwrap(), 0xCC);
    }

    #[test]
    fn test_stx_zero_page() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$CC
            0xA2, 0xCC, // STX
            0x86, 0x00, 0xEA, 0xEA,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        cpu.quick_step();

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "STX $00 = CC");
        assert_eq!(instruction_data.idle_cycles, 2);
        assert_eq!(cpu.bus.read(0x0000).unwrap(), 0xCC);
    }
}
