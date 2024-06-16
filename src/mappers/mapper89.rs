use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper89 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
}

impl Mapper89 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
    }
  }
}

impl Mapper for Mapper89 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x0000..=0xBFFF => {
        let bank = (self.bank_select & 0x70) >> 4;
        (bank as u32) * 0x4000 + (address as u32 & 0x3FFF)
      },
      0xC000..=0xFFFF => {
        (self.prg_rom_banks as u32 - 1) * 0x4000 + (address as u32 & 0x3FFF)
      },
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    if address <= 0x1FFF {
      ((self.bank_select & 0x7) | (self.bank_select & 0x80) >> 4) as u32 * 0x2000 + (address as u32 & 0x1FFF)
    } else {
      0
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x8000 {
      self.bank_select = value;
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    if self.bank_select & 0x8 != 0 {
      MirroringMode::SingleScreenHigh
    } else {
      MirroringMode::SingleScreenLow
    }
  }

  fn scanline(&mut self) {}

  fn irq_state(&self) -> bool {
    false
  }
}