use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper4 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
}

impl Mapper4 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
    }
  }
}

impl Mapper for Mapper4 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    todo!()
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    todo!()
  }

  fn mapped_cpu_write(&mut self, _address: u16, _value: u8) {}

  fn mirroring_mode(&self) -> MirroringMode {
    todo!()
  }
}