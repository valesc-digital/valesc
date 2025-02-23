use std::io::{Read, Seek};
use std::io;
use thiserror::Error;

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
    pub fn from_read<R: Read + Seek>(reader: &mut R) -> Result<InesFile, InesFileError>
    {
        println!("Parsing iNES ROM");

        let mut magic_bytes = [0; 4];
        reader.read_exact(&mut magic_bytes)?;

        // `0x1A` is the `SUB` (substitude) character
        if magic_bytes != *b"NES\x1A" {
            return Err(InesFileError::MagicBytesMissing);
        }

        println!("iNES magic characters are present");

        let mut prg_rom_size: [u8; 1] = [0; 1];
        reader.read_exact(&mut prg_rom_size)?;

        let prg_rom_size =  prg_rom_size[0] as usize * 16 * BYTES_ON_KIBIBYTE;
        println!("PRG ROM SIZE:{prg_rom_size}");

        let mut prg_rom = vec![0u8; prg_rom_size];
        
        reader.seek(io::SeekFrom::Start(16))?;
        reader.read_exact(&mut prg_rom)?;

        Ok(Self {
            prg_rom,
            prg_rom_size,
        })
    }
}