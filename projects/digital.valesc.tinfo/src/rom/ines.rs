use std::io::{Read, Seek};
use std::io;
use log::debug;
use thiserror::Error;

use crate::cartridge::nrom::Nrom;
use crate::cartridge::Cartridge;
use crate::rom::Rom;

pub const BYTES_ON_KIBIBYTE: usize = 1024;

pub struct InesFile {
    pub prg_rom: Vec<u8>,
    pub prg_rom_size: usize,
}

#[derive(Debug, Error)]
pub enum InesFileError {
    #[error("The iNES ROM is missing the magic bytes NES<SUB> at its start")]
    MagicBytesMissing,

    #[error("Unable to read the iNES ROM: {0}")]
    ReadingRomFailed(#[from] io::Error),
}

impl InesFile {
    pub fn from_read<R: Read + Seek>(reader: &mut R) -> Result<Box<dyn Cartridge>, InesFileError>
    {
        debug!("Parsing iNES ROM");

        let mut magic_bytes = [0; 4];
        reader.read_exact(&mut magic_bytes)?;

        // `0x1A` is the `SUB` (substitude) character
        if magic_bytes != *b"NES\x1A" {
            return Err(InesFileError::MagicBytesMissing);
        }

        debug!("iNES magic characters are present");

        let mut prg_rom_size: [u8; 1] = [0; 1];
        reader.read_exact(&mut prg_rom_size)?;

        let prg_rom_size =  prg_rom_size[0] as usize * 16 * BYTES_ON_KIBIBYTE;
        debug!("PRG ROM SIZE:{prg_rom_size}");

        let mut prg_rom = vec![0u8; prg_rom_size];
        
        reader.seek(io::SeekFrom::Start(16))?;
        reader.read_exact(&mut prg_rom)?;

        let rom = Self {
            prg_rom,
            prg_rom_size,
        };

        Ok(Box::new(Nrom::new(false, rom)))
    }
}

impl Rom for InesFile {
    fn read_prg_data(&self, index: usize) -> u8 {
        return self.prg_rom[index]
    }
}