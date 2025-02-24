use thiserror::Error;

pub(crate) mod nrom;

pub(crate) trait Cartridge {
    fn read(&self, address: u16) -> Result<u8, CartridgeError>;
    fn write(&mut self, _address: u16, _value: u8) -> Result<(), CartridgeError>;
}

#[derive(Error, Debug)]
pub(crate) enum CartridgeError {
    #[error("Unable to read data from the cartridge: {0}")]
    CannotRead(&'static str),

    #[error("Unable to read data from the cartridge: {0}")]
    CannotWrite(&'static str)
}