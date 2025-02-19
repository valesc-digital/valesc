// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::fs::File;
use tinfo::ines::InesFile;
use text_io::read;

pub(crate) struct Cpu {
    register_a: u8,
    register_x: u8,
    register_y: u8,

    status: u8,
    stack_pointer: u8,
    program_counter: u16
}

impl Cpu {
    pub(crate) fn new(ines: InesFile) -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,

            status: 0,
            stack_pointer: 0,
            program_counter: 0x0C000,
        }
    }

    pub(crate) fn step(&mut self) {
        println!("CPU Step")
    }
}

fn main() {
    let mut rom = File::open("nestest.nes").unwrap();
    let ines = InesFile::from_read(&mut rom).unwrap();
    
    let mut cpu = Cpu::new(ines);

    loop {
        let _: String = read!("{}\n");

        cpu.step();
    }
}