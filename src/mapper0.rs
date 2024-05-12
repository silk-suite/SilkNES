use crate::mapper::Mapper;

pub struct Mapper0 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
}

impl Mapper0 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
    }
  }
}

impl Mapper for Mapper0 {
  fn get_mapped_address_cpu(&self, address: u16) -> u16 {
    if address >= 0x8000 {
      let mask = if self.prg_rom_banks > 1 { 0x7FFF } else { 0x3FFF };
      return (address & mask) as u16;
    } else {
      0
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u16 {
    if address <= 0x1FFF {
      address
    } else {
      panic!("Tried to get mapped address for: {:04X}", address);
    }
  }
}