//! Holds the implementation of the `LDX` instruction.

use crate::bus::BusError;
use crate::cpu::Cpu;
use crate::cpu::CycleError;
use crate::{build_address, cpu::impl_instruction_cycles};
use crate::cpu::InstructionData;


impl Cpu {
    /// Implements the immediate load X register instruction data.
    pub(super) fn load_x_register_immediate_instruction(&mut self) -> Result<InstructionData, BusError> {
        let arg_1 = self.bus.read(self.program_counter + 1)?;

        Ok(InstructionData {
            arg_1: Some(arg_1),
            arg_2: None,
            assembly: format!("LDX #${arg_1:02X}"),
            idle_cycles: 1,
        })
    }
}

impl_instruction_cycles!(
    /// Implements the immediate load X register instruction cycles.
    cpu, load_x_register_immediate_cycles,

    2, true => {
        cpu.register_x = cpu.read_program_counter()?;
        cpu.program_counter += 1;
        cpu.set_signedness(cpu.register_x);
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::{tests::*, CpuStatusFlags};

    #[test]
    fn test_ldx_immediate_positive() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$5C
            0xA2, 0x5C
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "LDX #$5C");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
        assert_eq!(cpu.register_x, 0x5C);
        assert!(!cpu.status.contains(CpuStatusFlags::Zero));
        assert!(!cpu.status.contains(CpuStatusFlags::Negative));
    }

    #[test]
    fn test_ldx_immediate_negative() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$FC
            0xA2, 0xFC
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "LDX #$FC");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
        assert_eq!(cpu.register_x, 0xFC);
        assert!(!cpu.status.contains(CpuStatusFlags::Zero));
        assert!(cpu.status.contains(CpuStatusFlags::Negative));
    }

    #[test]
    fn test_ldx_immediate_zero() {
        let cartridge = MockCartridge::new(vec![
            // LDX #$00
            0xA2, 0x00
        ]);

        let mut cpu = Cpu::new(Box::new(cartridge));

        let instruction_data = cpu.cycle().unwrap().unwrap().instruction_data;
        assert_eq!(instruction_data.assembly, "LDX #$00");
        assert_eq!(instruction_data.idle_cycles, 1);

        assert_eq!(cpu.program_counter, 0x8001);

        cpu.cycle().unwrap();
        assert_eq!(cpu.program_counter, 0x8002);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.contains(CpuStatusFlags::Zero));
        assert!(!cpu.status.contains(CpuStatusFlags::Negative));
    }
}