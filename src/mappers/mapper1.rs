use crate::mapper::Mapper;

#[derive(Debug, Clone, Copy)]
pub struct MMC1Registers {
  shift_register: u16,
  control_register: u8,
  chr_bank_0: u8,
  chr_bank_1: u8,
  prg_bank: u8,
  shift_register_writes: u8,
}

impl Default for MMC1Registers {
  fn default() -> Self {
    Self {
      shift_register: 0,
      control_register: 0xC,
      chr_bank_0: 0,
      chr_bank_1: 0,
      prg_bank: 0,
      shift_register_writes: 0,
    }
  }
}

pub struct Mapper1 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  registers: MMC1Registers,
}

impl Mapper1 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      registers: MMC1Registers::default(),
    }
  }
}

impl Mapper for Mapper1 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x6000..=0x7FFF => address as u32,
      0x8000..=0xFFFF => {
        let bank_mode = (self.registers.control_register & 0b1100) >> 2;
        match (address, bank_mode) {
          (0x8000..=0xBFFF, 0 | 1) | (0xC000..=0xFFFF, 0 | 1) => {
            // switch 32 KB at $8000, ignoring low bit of bank number
            ((self.registers.prg_bank & 0xE) as u32 * 0x8000) + (address & 0x7FFF) as u32
          },
          (0x8000..=0xBFFF, 2) => {
            // fix first bank at $8000 and switch 16 KB bank at $C000
            (address % 0x4000) as u32
          },
          (0xC000..=0xFFFF, 2) | (0x8000..=0xBFFF, 3) => {
            // fix last bank at $C000 and switch 16 KB bank at $8000
            ((self.registers.prg_bank & 0xF) as u32 * 0x4000) + (address & 0x3FFF) as u32
          },
          (0xC000..=0xFFFF, 3) => {
            // fix last bank at $C000 and switch 16 KB bank at $8000
            ((self.prg_rom_banks - 1) as u32 * 0x4000) + (address & 0x3FFF) as u32
          },
          _ => panic!("Invalid prg rom bank mode for MMC1: {}", bank_mode),
        }
      }
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    let is_8k_mode = self.registers.control_register & 0b10000 == 0;
    match address {
      0x0000..=0x0FFF => {
        if is_8k_mode {
          (self.registers.chr_bank_0 as u32 * 0x2000) + (address & 0x1FFF) as u32
        } else {
          (self.registers.chr_bank_0 as u32 * 0x1000) + (address & 0x0FFF) as u32
        }
      },
      0x1000..=0x1FFF => {
        if is_8k_mode {
          (self.registers.chr_bank_0 as u32 * 0x2000) + (address & 0x1FFF) as u32
        } else {
          (self.registers.chr_bank_1 as u32 * 0x1000) + (address & 0x0FFF) as u32
        }
      },
      _ => 0,
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    let shift_bit = value as u16 & 0x1;
    if value & 0x80 != 0 {
      self.registers.shift_register = 0;
      self.registers.shift_register_writes = 0;
      self.registers.control_register |= 0x0C;
    } else {
      self.registers.shift_register >>= 1;
      self.registers.shift_register |= shift_bit << 4;
      self.registers.shift_register_writes += 1;
    }

    if self.registers.shift_register_writes == 5 {
      let target_register = (address >> 13) & 0x03;
      match target_register {
        0 => {
          self.registers.control_register = self.registers.shift_register as u8 & 0x1F;
        },
        1 => {
          self.registers.chr_bank_0 = self.registers.shift_register as u8 & if self.registers.control_register & 0b10000 != 0 { 0x1F } else { 0x1E };
        },
        2 => {
          self.registers.chr_bank_1 = self.registers.shift_register as u8 & 0x1F;
        },
        3 => {
          self.registers.prg_bank = self.registers.shift_register as u8 & 0x1F;
        },
        _ => {}
      }
      self.registers.shift_register = 0x0;
      self.registers.shift_register_writes = 0;
    }
  }

  fn mirroring_mode(&self) -> crate::cartridge::MirroringMode {
      match (self.registers.control_register & 0b10000) >> 4 {
        0 => crate::cartridge::MirroringMode::SingleScreenLow,
        1 => crate::cartridge::MirroringMode::SingleScreenHigh,
        2 => crate::cartridge::MirroringMode::Vertical,
        3 => crate::cartridge::MirroringMode::Horizontal,
        _ => panic!("Invalid mirroring mode for MMC1: {}", (self.registers.control_register & 0b10000) >> 4),
      }
  }
}