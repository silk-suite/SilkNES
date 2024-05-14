use crate::mapper::Mapper;

pub struct Mapper2 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
}

impl Mapper2 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
    }
  }
}

impl Mapper for Mapper2 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x8000..=0xBFFF => {
        (self.bank_select as u32 * 0x4000) + (address & 0x3FFF) as u32
      },
      0xC000..=0xFFFF => {
        ((self.prg_rom_banks - 1) as u32 * 0x4000) + (address & 0x3FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    if address <= 0x1FFF {
      address as u32
    } else {
      panic!("Tried to get mapped address for: {:04X}", address);
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x8000 {
      self.bank_select = value & 0xF;
    }
  }
}