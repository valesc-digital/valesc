pub(crate) mod ines;

pub(crate) trait Rom {
    fn read_prg_data(&self, index: usize) -> u8;
} 