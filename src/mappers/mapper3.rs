use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper3 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
}

impl Mapper3 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
    }
  }
}

impl Mapper for Mapper3 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    if address >= 0x8000 {
      let mask = if self.prg_rom_banks > 1 { 0x7FFF } else { 0x3FFF };
      return (address & mask) as u32;
    } else {
      0
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    if address <= 0x1FFF {
      (0x2000 * self.bank_select as u32) + address as u32
    } else {
      0
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x8000 {
      self.bank_select = value & 0xF;
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    MirroringMode::_Hardwired
  }
}