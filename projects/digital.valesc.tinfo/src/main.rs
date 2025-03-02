//! Experimental runner for the library.
//!
//! # TODO
//! This file/crate should be removed when a working GUI implementation of the emulator is available.

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use std::fs::File;
use std::io::Write;

use env_logger::fmt::style::{AnsiColor, Style};
use env_logger::Env;
use tinfo::cpu::Cpu;
use tinfo::rom::ines::InesFile;

fn main() {
    // Set the minimum log level to `warn`
    // TRACK: https://github.com/rust-cli/env_logger/issues/162
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format(|buf, record| {
            let bold_red_style = Style::new().bold().fg_color(Some(AnsiColor::Red.into()));
            let bold_cyan_style = Style::new().bold().fg_color(Some(AnsiColor::Cyan.into()));
            let bold_green_style = Style::new().bold().fg_color(Some(AnsiColor::Green.into()));
            let bold_yellow_style = Style::new().bold().fg_color(Some(AnsiColor::Yellow.into()));
            let bold_magenta_style = Style::new()
                .bold()
                .fg_color(Some(AnsiColor::Magenta.into()));

            let header = match record.level() {
                log::Level::Trace => format!("[ {bold_magenta_style}TRACE{bold_magenta_style:#} ]"),
                log::Level::Debug => format!("[ {bold_cyan_style}DEBUG{bold_cyan_style:#} ]"),
                log::Level::Info => format!("[ {bold_green_style}INFO{bold_green_style:#} ] "),
                log::Level::Warn => format!("[ {bold_yellow_style}WARN{bold_yellow_style:#} ] "),
                log::Level::Error => format!("[ {bold_red_style}ERROR{bold_red_style:#} ]"),
            };

            writeln!(buf, "{header} {}", record.args())
        })
        .init();

    let mut rom_file = File::open("nestest.nes").unwrap();
    let cartridge = InesFile::from_read(&mut rom_file).unwrap();

    let mut cpu = Cpu::new_with_program_counter(cartridge, 0xC000);

    loop {
        if let Some(cpu_snapshot) = cpu.cycle().unwrap() {
            let log_padding = " ".repeat(32 - cpu_snapshot.instruction_data.assembly.len());

            println!(
                "{:04X}  {:02X} {} {}  {}{log_padding}A:{:02X} X:{:02X} Y:{:02X} P:{:02} SP:{:02X} PPU:  0,  0 CYC:{}",
                cpu_snapshot.program_counter,
                cpu_snapshot.opcode,
                cpu_snapshot.instruction_data.arg_1.map(|arg| format!("{arg:02X}")).unwrap_or(String::from("  ")),
                cpu_snapshot.instruction_data.arg_2.map(|arg| format!("{arg:02X}")).unwrap_or(String::from("  ")),
                cpu_snapshot.instruction_data.assembly,
                cpu_snapshot.accumulator,
                cpu_snapshot.register_x,
                cpu_snapshot.register_y,
                cpu_snapshot.status,
                cpu_snapshot.stack_pointer,
                cpu_snapshot.cpy_cycles
            );
        }
    }
}
