use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper152 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
}

impl Mapper152 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
    }
  }
}

impl Mapper for Mapper152 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x8000..=0xBFFF => {
        (((self.bank_select as u32 >> 4) & 0x7) * 0x4000) + (address & 0x3FFF) as u32
      },
      0xC000..=0xFFFF => {
        ((self.prg_rom_banks as u32 - 1) * 0x4000) + (address & 0x3FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    if address <= 0x1FFF {
      ((self.bank_select as u32 & 0xF) * 0x2000) + address as u32
    } else {
      panic!("Tried to get mapped address for: {:04X}", address);
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x8000 {
      self.bank_select = value;
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    if self.bank_select & 0x80 != 0 {
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