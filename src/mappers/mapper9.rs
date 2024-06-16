use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper9 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  prg_rom_bank: u8,
  chr_rom_bank_1: u8,
  chr_rom_bank_2: u8,
  chr_rom_bank_3: u8,
  chr_rom_bank_4: u8,
  mirroring: bool,
}

impl Mapper9 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      prg_rom_bank: 0,
      chr_rom_bank_1: 0,
      chr_rom_bank_2: 0,
      chr_rom_bank_3: 0,
      chr_rom_bank_4: 0,
      mirroring: false,
    }
  }
}

impl Mapper for Mapper9 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x8000..=0x9FFF => {
        ((self.prg_rom_bank as u32) * 0x2000) + (address & 0x1FFF) as u32
      },
      0xA000..=0xBFFF => {
        (self.prg_rom_banks as u32 - 3) * 0x2000 + (address & 0x1FFF) as u32
      },
      0xC000..=0xDFFF => {
        (self.prg_rom_banks as u32 - 2) * 0x2000 + (address & 0x1FFF) as u32
      },
      0xE000..=0xFFFF => {
        (self.prg_rom_banks as u32 - 1) * 0x2000 + (address & 0x1FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    match address {
      0x0000..=0x0FFF => {
        (self.chr_rom_bank_1 as u32 * 0x1000) + (address & 0x0FFF) as u32
      },
      0x1000..=0x1FFF => {
        (self.chr_rom_bank_3 as u32 * 0x1000) + (address & 0x0FFF) as u32
      },
      _ => 0,
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    match address {
      0xA000..=0xAFFF => {
        self.prg_rom_bank = value & 0xF;
      },
      0xB000..=0xBFFF => {
        self.chr_rom_bank_1 = value & 0x1F;
      },
      0xC000..=0xCFFF => {
        self.chr_rom_bank_2 = value & 0x1F;
      },
      0xD000..=0xDFFF => {
        self.chr_rom_bank_3 = value & 0x1F;
      },
      0xE000..=0xEFFF => {
        self.chr_rom_bank_4 = value & 0x1F;
      },
      0xF000..=0xFFFF => {
        self.mirroring = value & 1 == 1;
      },
      _ => {},
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    if self.mirroring {
      MirroringMode::Horizontal
    } else {
      MirroringMode::Vertical
    }
  }

  fn scanline(&mut self) {}

  fn irq_state(&self) -> bool {
    false
  }
}