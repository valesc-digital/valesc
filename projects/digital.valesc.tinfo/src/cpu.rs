//! Holds the implementation of the modified 6502 CPU used by the NES.

use std::{fs::File, ptr::addr_of, sync::Arc};
use text_io::read;
use bitflags::bitflags;

use crate::{bus::Bus, cartridge::{self, Cartridge}};

const CPU_RAM_START_ADDRESS: u16 = 0x0000;
const CPU_RAM_END_ADDRESS_AFTER_MIRRORS: u16 = 0x1FFF;

bitflags! {
    // Attributes can be applied to flags types
    #[derive(Debug)]
    pub struct CpuStatusFlags: u8 {
        const Carry = 0b00000001;
        const Zero = 0b00000010;
        const InterruptsDisabled = 0b00000100;
        const Decimal = 0b00001000;
        const B = 0b00010000; // CHECK: Better name? & B flag support (https://www.nesdev.org/wiki/Status_flags#The_B_flag)
        const Stub = 0b00100000; // Does noting, always 0b1
        const Overflow = 0b01000000;
        const Negative = 0b10000000;
    }
}
pub struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,

    status: CpuStatusFlags,
    stack_pointer: u8,
    program_counter: u16,

    bus: Bus,
}

impl Cpu {
    pub fn new(cartridge: Box<dyn Cartridge>) -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,

            status: CpuStatusFlags::Decimal | CpuStatusFlags::B,
            stack_pointer: 0xFD, // CHECK: Why 0xFD and not 0xFF
            //program_counter: 0x8000,
            program_counter: 0xC000,

            bus: Bus::new(cartridge),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.bus.read(self.program_counter).unwrap();
        let arg_1 = self.bus.read(self.program_counter + 1).unwrap();
        let arg_2 = self.bus.read(self.program_counter + 2).unwrap();

        println!("{:X}  {opcode:02X} {arg_1:02X} {arg_2:02X}    A:{:02X} X:{:02X} Y:{:02X} P:{:02} SP:{:02X}", self.program_counter, self.register_a, self.register_x, self.register_y, self.status.bits(), self.stack_pointer);

        match opcode {
            0x4C => self.jump_absolute(arg_1, arg_2),
            0xA2 => self.load_x_register_immediate(arg_1),
            0x86 => self.store_x_register_zero_page(arg_1),
            0x20 => self.jump_to_subroutine(arg_1, arg_2),
            0x38 => self.set_carry_flag(),
            0x18 => self.clear_carry_flag(),
            0xB0 => self.branch_if_carry_set(arg_1),
            0x90 => self.branch_if_carry_clear(arg_1),
            0xEA => self.no_operation(),
            0xA9 => self.load_accumulator_immediate(arg_1),
            0xF0 => self.branch_if_equal(arg_1),
            _ => unimplemented!()
        }
    }

    fn jump_absolute(&mut self, arg_1: u8, arg_2: u8) {
        self.program_counter = (arg_1 as u16) | ((arg_2 as u16) << 8);
    }

    fn load_x_register_immediate(&mut self, arg_1: u8) {
        self.register_x = arg_1;

        if arg_1 == 0 {
            self.status |= CpuStatusFlags::Zero;
        } else if (arg_1 as i8) < 0 {
            self.status -= CpuStatusFlags::Zero;
            self.status |= CpuStatusFlags::Negative;
        }

        self.program_counter += 2;
    }

    fn store_x_register_zero_page(&mut self, arg_1: u8) {
        self.bus.write(arg_1 as u16, self.register_x);

        self.program_counter += 2;
    }

    fn jump_to_subroutine(&mut self, arg_1: u8, arg_2: u8) {
        let return_adress = self.program_counter + 3 - 1;
        self.jump_absolute(arg_1, arg_2);

        self.bus.write(0x100 + self.stack_pointer as u16, (return_adress & 0x00FF) as u8);
        self.stack_pointer -= 1;

        self.bus.write(0x100 + self.stack_pointer as u16, ((return_adress & 0xFF00 ) >> 8) as u8);
        self.stack_pointer -= 1;
    }

    fn branch_if_carry_set(&mut self, arg_1: u8) {
        if !self.status.contains(CpuStatusFlags::Carry) {
            self.program_counter += 2;
            return;
        }

        self.program_counter += arg_1 as u16;
    }

    fn branch_if_carry_clear(&mut self, arg_1: u8) {
        if self.status.contains(CpuStatusFlags::Carry) {
            self.program_counter += 2;
            return;
        }

        self.program_counter += arg_1 as u16;
    }

    fn set_carry_flag(&mut self) {
        self.status |= CpuStatusFlags::Carry;
        self.program_counter += 1;
    }

    fn clear_carry_flag(&mut self) {
        self.status -= CpuStatusFlags::Carry;
        self.program_counter += 1;
    }

    fn no_operation(&mut self) {
        self.program_counter += 1;
    }

    fn load_accumulator_immediate(&mut self, arg_1: u8) {
        self.register_a = arg_1;

        if arg_1 == 0 {
            self.status |= CpuStatusFlags::Zero;
        } else {
            self.status -= CpuStatusFlags::Zero;
        }

        if (arg_1 as i8) < 0 {
            self.status |= CpuStatusFlags::Negative;
        } else {
            self.status -= CpuStatusFlags::Negative;
        }

        self.program_counter += 2;
    }

    fn branch_if_equal(&mut self, arg_1: u8) {
        if !self.status.contains(CpuStatusFlags::Zero) {
            self.program_counter += 2;
            return;
        }

        self.program_counter += arg_1 as u16;
    }
}