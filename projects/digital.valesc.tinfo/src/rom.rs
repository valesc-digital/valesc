//! Holds implementations to retrieve the static ROM data of a NES cartridge. 

pub mod ines;

/// The [Rom] trait provides a way to access the static data hold in
/// the ROM chips of a NES cartridge.
/// 
/// See also: [crate::cartridge::Cartridge]
pub(crate) trait Rom {
    /// Get a byte from the PRG ROM data chip, all banks should be merge and globally
    /// accessible by an index by concatenating them.
    fn read_prg_data(&self, index: usize) -> u8;
} 