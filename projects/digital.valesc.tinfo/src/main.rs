//! Experimental runner for the library.
//! 
//! # TODO
//! This file/crate should be removed when a working GUI implementation of the emulator is available.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use tinfo::rom::ines::InesFile;
use tinfo::cpu::Cpu;
use std::fs::File;

fn main() {
    let mut rom_file = File::open("nestest.nes").unwrap();
    let cartridge = InesFile::from_read(&mut rom_file).unwrap();

    let mut cpu = Cpu::new(cartridge);

    loop {
        cpu.step();
    }
}