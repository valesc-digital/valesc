//! Holds the implementation of the `LDX` instruction.

use crate::cpu::{Cpu, CpuError, InstructionData};

impl Cpu {
    /// Load a value to the register X using an immediate value.
    pub(super) fn load_x_register_immediate(
        &mut self,
        arg_1: u8,
    ) -> Result<InstructionData, CpuError> {
        self.register_x = arg_1;
        self.set_signedness(arg_1);

        Ok(InstructionData {
            idle_cycles: 1,
            assembly: format!("LDX #${arg_1:02X}"),
            increase_program_counter: 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{tests::*, CpuStatusFlags};

    #[test]
    fn test_ldx_immediate_positive() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$5C
            0xA2, 0x5C, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        cpu.status |= CpuStatusFlags::Negative | CpuStatusFlags::Zero;

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "LDX #$5C");
        assert_eq!(instruction_data.idle_cycles, 1);
        assert_eq!(cpu.register_x, 0x5C);

        assert!(!cpu.status.contains(CpuStatusFlags::Negative));
        assert!(!cpu.status.contains(CpuStatusFlags::Zero));
    }

    #[test]
    fn test_ldx_immediate_zero() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$00
            0xA2, 0x00, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        // Put the CPU to an impossible state only to check if
        // the flags are always corrected
        cpu.status |= CpuStatusFlags::Negative;
        cpu.status -= CpuStatusFlags::Zero;

        // Zero value
        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "LDX #$00");
        assert_eq!(instruction_data.idle_cycles, 1);
        assert_eq!(cpu.register_x, 0x00);

        assert!(!cpu.status.contains(CpuStatusFlags::Negative));
        assert!(cpu.status.contains(CpuStatusFlags::Zero));
    }

    #[test]
    fn test_ldx_immediate_negative() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$81
            0xA2, 0x81, NOP,
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        cpu.status |= CpuStatusFlags::Zero;
        cpu.status -= CpuStatusFlags::Negative;

        let instruction_data = cpu.quick_step();

        assert_eq!(instruction_data.assembly, "LDX #$81");
        assert_eq!(instruction_data.idle_cycles, 1);
        assert_eq!(cpu.register_x, 0x81);

        assert!(cpu.status.contains(CpuStatusFlags::Negative));
        assert!(!cpu.status.contains(CpuStatusFlags::Zero));
    }
}
