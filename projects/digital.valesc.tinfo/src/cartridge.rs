//! Holds the implementation of different types of cartridges that
//! has been used on the NES.

use thiserror::Error;

pub(crate) mod nrom;

/// The [Cartridge] trait provides an implementation of the hardware of a NES cartridge,
/// both in its static and dynamic behaviors.
/// 
/// Usually a cartridge will only store ROM data and emulate a mapper chip.
/// 
/// See also: [crate::rom::Rom].
pub trait Cartridge {
    /// Read data from the cartridge.
    /// 
    /// # Safety
    /// The given `address` is relative to the NES CPU global memory map,
    /// calls below `0x4020` may not be handled by the implementor.
    unsafe fn read(&self, address: u16) -> Result<u8, CartridgeError>;

    /// Write data to the cartridge.
    /// 
    /// # Safety
    /// The given `address` is relative to the NES CPU global memory map,
    /// calls below `0x4020` may not be handled by the implementor.
    unsafe fn write(&mut self, _address: u16, _value: u8) -> Result<(), CartridgeError>;
}

#[derive(Error, Debug)]
/// Errors that may happens when interacting with a cartridge.
pub enum CartridgeError {
    #[error("Unable to read data from the cartridge: {0}")]
    /// Unable to read data from the cartridge.
    CannotRead(&'static str),

    #[error("Unable to read data from the cartridge: {0}")]
    /// Unable to read data from the cartridge.
    CannotWrite(&'static str)
}