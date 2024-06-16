use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

#[derive(Debug, Default, Clone, Copy)]
pub struct NAMCOT3446Registers {
  pub prg_bank_1: u8,
  pub prg_bank_2: u8,
  pub chr_bank_1: u8,
  pub chr_bank_2: u8,
  pub chr_bank_3: u8,
  pub chr_bank_4: u8,
}

pub struct Mapper76 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
  bank_data: u8,
  registers: NAMCOT3446Registers,
}

impl Mapper76 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
      bank_data: 0,
      registers: NAMCOT3446Registers::default(),
    }
  }
}

impl Mapper for Mapper76 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x8000..=0x9FFF => {
        ((self.registers.prg_bank_1 as u32) * 0x2000) + (address & 0x1FFF) as u32
      },
      0xA000..=0xBFFF => {
        ((self.registers.prg_bank_2 as u32) * 0x2000) + (address & 0x1FFF) as u32
      },
      0xC000..=0xDFFF => {
        ((self.prg_rom_banks as u32 * 2 - 2) * 0x2000) + (address & 0x1FFF) as u32
      },
      0xE000..=0xFFFF => {
        ((self.prg_rom_banks as u32 * 2 - 1) * 0x2000) + (address & 0x1FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    match address {
      0x0000..=0x07FF => {
        ((self.registers.chr_bank_1 as u32) * 0x0800) + (address & 0x07FF) as u32
      },
      0x0800..=0x0FFF => {
        ((self.registers.chr_bank_2 as u32) * 0x0800) + (address & 0x07FF) as u32
      },
      0x1000..=0x1BFF => {
        ((self.registers.chr_bank_3 as u32) * 0x0800) + (address & 0x07FF) as u32
      },
      0x1C00..=0x1FFF => {
        ((self.registers.chr_bank_4 as u32) * 0x0800) + (address & 0x07FF) as u32
      },
      _ => 0,
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    match address {
      0x8000 => {
        self.bank_select = value & 0x7;
      },
      0x8001 => {
        let data = value & 0x3F;
        match self.bank_select {
          0 => self.registers.chr_bank_1 = data,
          1 => self.registers.chr_bank_2 = data,
          2 => self.registers.chr_bank_3 = data,
          3 => self.registers.chr_bank_4 = data,
          4 => self.registers.prg_bank_1 = data,
          5 => self.registers.prg_bank_2 = data,
          _ => {},
        }
      },
      _ => {},
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    MirroringMode::_Hardwired
  }

  fn scanline(&mut self) {}

  fn irq_state(&self) -> bool {
    false
  }
}