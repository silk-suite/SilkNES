use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

#[derive(Debug, Default, Clone, Copy)]
pub struct MMC3Registers {
  /// 2 KB CHR bank at PPU $0000-$07FF (or $1000-$17FF)
  r0: u8,
  /// 2 KB CHR bank at PPU $0800-$0FFF (or $1800-$1FFF)
  r1: u8,
  /// 1 KB CHR bank at PPU $1000-$13FF (or $0000-$03FF)
  r2: u8,
  /// 1 KB CHR bank at PPU $1400-$17FF (or $0400-$07FF)
  r3: u8,
  /// 1 KB CHR bank at PPU $1800-$1BFF (or $0800-$0BFF)
  r4: u8,
  /// 1 KB CHR bank at PPU $1C00-$1FFF (or $0C00-$0FFF)
  r5: u8,
  /// 8 KB PRG ROM bank at $8000-$9FFF (or $C000-$DFFF)
  r6: u8,
  /// 8 KB PRG ROM bank at $A000-$BFFF
  r7: u8,
  bank_select: u8,
  mirroring_mode: bool,
  irq_latch: u8,
  irq_enabled: bool,
  irq_active: bool,
  irq_counter: u8,
}

pub struct Mapper4 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  registers: MMC3Registers,
}

impl Mapper4 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      registers: MMC3Registers::default(),
    }
  }
}

impl Mapper for Mapper4 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    let prg_rom_bank_mode = (self.registers.bank_select & 0b0100_0000) >> 6;
    match (address, prg_rom_bank_mode) {
      (0x6000..=0x7FFF, _) => {
        address as u32
      },
      (0x8000..=0x9FFF, 0) => {
        (self.registers.r6 as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      (0x8000..=0x9FFF, 1) => {
        (((self.prg_rom_banks * 2) - 2) as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      (0xA000..=0xBFFF, _) => {
        (self.registers.r7 as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      (0xC000..=0xDFFF, 0) => {
        (((self.prg_rom_banks * 2) - 2) as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      (0xC000..=0xDFFF, 1) => {
        (self.registers.r6 as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      (0xE000..=0xFFFF, _) => {
        (((self.prg_rom_banks * 2) - 1) as u32 * 0x2000) + (address & 0x1FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    let chr_rom_bank_mode = (self.registers.bank_select & 0b1000_0000) >> 7;
    match (address, chr_rom_bank_mode) {
      (0x0000..=0x03FF, 0) | (0x1000..=0x13FF, 1) => {
        (self.registers.r0 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      (0x0400..=0x07FF, 0) | (0x1400..=0x17FF, 1) => {
        (self.registers.r0 as u32 * 0x400) + 0x400 + (address & 0x3FF) as u32
      },
      (0x0800..=0x0BFF, 0) | (0x1800..=0x1BFF, 1) => {
        (self.registers.r1 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      (0x0C00..=0x0FFF, 0) | (0x1C00..=0x1FFF, 1) => {
        (self.registers.r1 as u32 * 0x400) + 0x400 + (address & 0x3FF) as u32
      },
      (0x0000..=0x03FF, 1) | (0x1000..=0x13FF, 0) => {
        (self.registers.r2 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      (0x0400..=0x07FF, 1) | (0x1400..=0x17FF, 0) => {
        (self.registers.r3 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      (0x0800..=0x0BFF, 1) | (0x1800..=0x1BFF, 0) => {
        (self.registers.r4 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      (0x0C00..=0x0FFF, 1) | (0x1C00..=0x1FFF, 0) => {
        (self.registers.r5 as u32 * 0x400) + (address & 0x3FF) as u32
      },
      _ => 0,
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    let even = address % 2 == 0;
    match (address, even) {
      (0x8000..=0x9FFF, true) => {
        self.registers.bank_select = value;
      }
      (0x8000..=0x9FFF, false) => {
        let bank = self.registers.bank_select & 0b0000_0111;
        match bank {
          0 => self.registers.r0 = value,
          1 => self.registers.r1 = value,
          2 => self.registers.r2 = value,
          3 => self.registers.r3 = value,
          4 => self.registers.r4 = value,
          5 => self.registers.r5 = value,
          6 => self.registers.r6 = value & 0b0011_1111,
          7 => self.registers.r7 = value & 0b0011_1111,
          _ => unreachable!(),
        }
      },
      (0xA000..=0xBFFF, true) => {
        self.registers.mirroring_mode = value & 0b1 == 1;
      }
      (0xA000..=0xBFFF, false) => {
        // TODO: PRG RAM PROTECT
      }
      (0xC000..=0xDFFF, true) => {
        self.registers.irq_latch = value;
      }
      (0xC000..=0xDFFF, false) => {
        self.registers.irq_counter = self.registers.irq_latch;
      }
      (0xE000..=0xFFFF, true) => {
        self.registers.irq_enabled = false;
        self.registers.irq_active = false;
      }
      (0xE000..=0xFFFF, false) => {
        self.registers.irq_enabled = true;
      }
      _ => {}
    }
  }

  fn mirroring_mode(&self) -> MirroringMode {
    if self.registers.mirroring_mode {
      MirroringMode::Horizontal
    } else {
      MirroringMode::Vertical
    }
  }

  fn scanline(&mut self) {
    if self.registers.irq_counter == 0 {
      self.registers.irq_counter = self.registers.irq_latch;
    } else {
      self.registers.irq_counter -= 1;
    }

    if self.registers.irq_counter == 0 && self.registers.irq_enabled {
      self.registers.irq_active = true;
    }
  }

  fn irq_state(&self) -> bool {
    self.registers.irq_active
  }
}