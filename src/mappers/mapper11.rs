use crate::cartridge::MirroringMode;
use crate::mapper::Mapper;

pub struct Mapper11 {
  prg_rom_banks: u8,
  chr_rom_banks: u8,
  bank_select: u8,
}

impl Mapper11 {
  pub fn new(prg_rom_banks: u8, chr_rom_banks: u8) -> Self {
    Self {
      prg_rom_banks,
      chr_rom_banks,
      bank_select: 0,
    }
  }
}

impl Mapper for Mapper11 {
  fn get_mapped_address_cpu(&self, address: u16) -> u32 {
    match address {
      0x8000..=0xFFFF => {
        //println!("{}", address);
        ((self.bank_select as u32 & 0xF) * 0x8000) + (address & 0x7FFF) as u32
      },
      _ => 0,
    }
  }

  fn get_mapped_address_ppu(&self, address: u16) -> u32 {
    if address <= 0x1FFF {
      (((self.bank_select as u32 >> 4) & 0xF) * 0x2000) + address as u32
    } else {
      panic!("Tried to get mapped address for: {:04X}", address);
    }
  }

  fn mapped_cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x8000 {
      println!("Bank select: {:#08b}", value);
      self.bank_select = value;
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