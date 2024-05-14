use crate::mapper::Mapper;

pub struct Mapper1 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
}

impl Mapper1 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
    }
  }
}

impl Mapper for Mapper1 {
  fn get_mapped_address_cpu(&self, address: u16) -> u16 {
    todo!()
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u16 {
    todo!()
  }
}