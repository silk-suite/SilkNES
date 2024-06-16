use std::fmt::Debug;
use std::fs;
use std::path::Path;

use crate::mapper::Mapper;
use crate::mappers::{
  mapper0::Mapper0,
  mapper1::Mapper1,
  mapper2::Mapper2,
  mapper3::Mapper3,
  mapper4::Mapper4,
  mapper7::Mapper7,
};

pub struct Cartridge {
  pub header_info: HeaderInfo,
  pub mapper_id: u8,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub mapper: Box<dyn Mapper>,
  pub has_ram: bool,
  pub ram: Vec<u8>,
}

impl Cartridge {
  pub fn from_rom(rom_path: &str) -> Self {
    let bytes = fs::read(Path::new(rom_path)).expect(&format!("Failed to load ROM from supplied path: {}", rom_path));
    Cartridge::from_bytes(bytes)
  }

  pub fn from_bytes(rom_bytes: Vec<u8>) -> Self {
    match parse_header(&rom_bytes) {
      Ok(header_info) => {
        let mapper_id = (header_info.flags6 & 0b1111_0000) >> 4 | (header_info.flags7 & 0b1111_0000);
        let mapper = match mapper_id {
          0 => Box::new(Mapper0::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          1 => Box::new(Mapper1::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          2 => Box::new(Mapper2::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          3 => Box::new(Mapper3::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          4 => Box::new(Mapper4::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          7 => Box::new(Mapper7::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          _ => panic!("Mapper {} not implemented.", mapper_id),
        };
        let prg_start: u32 = 0x0010;
        let prg_end: u32 = prg_start + (0x4000 * header_info.prg_rom_size as u32);
        let chr_start: u32 = prg_end;
        let chr_end: u32 = chr_start + (0x2000 * header_info.chr_rom_size as u32);
        println!("PRG: {:#06X} - {:#06X}, CHR: {:#06X} - {:#06X}, Mapper: {}", prg_start, prg_end, chr_start, chr_end, mapper_id);
        let chr_rom = if header_info.chr_rom_size == 0 {
          vec![0; 0x2000]
        } else {
          rom_bytes[chr_start as usize..chr_end as usize].to_vec()
        };
        let has_ram = (header_info.flags6 & 0b0000_0010) != 0;
        Self {
          header_info,
          mapper_id,
          prg_rom: rom_bytes[prg_start as usize..prg_end as usize].to_vec(),
          chr_rom,
          mapper,
          has_ram,
          ram: vec![0; 0x8000],
        }
      },
      Err(_) => panic!("Failed to parse ROM from supplied bytes."),
    }
  }

  pub fn cpu_read(&self, address: u16) -> u8 {
    if self.has_ram && address >= 0x6000 && address <= 0x7FFF {
      self.ram[self.mapper.get_mapped_address_cpu(address) as usize]
    } else {
      self.prg_rom[self.mapper.get_mapped_address_cpu(address) as usize]
    }
  }

  pub fn cpu_write(&mut self, address: u16, value: u8) {
    if self.has_ram && address >= 0x6000 && address <= 0x7FFF {
      self.ram[self.mapper.get_mapped_address_cpu(address) as usize] = value
    } else {
      self.mapper.mapped_cpu_write(address, value);
    }
  }

  pub fn ppu_read(&self, address: u16) -> &u8 {
    let mapped_address = self.mapper.get_mapped_address_ppu(address) as usize;
    if (mapped_address) < self.chr_rom.len() {
      &self.chr_rom[mapped_address]
    } else {
      &0
    }
  }

  pub fn ppu_write(&mut self, address: u16, value: u8) {
    self.chr_rom[self.mapper.get_mapped_address_ppu(address) as usize] = value
  }

  pub fn get_nametable_layout(&self) -> MirroringMode {
    let mapper_mirroring_mode = self.mapper.mirroring_mode();
    if mapper_mirroring_mode == MirroringMode::_Hardwired {
      if self.header_info.flags6 & 0b0000_0001 == 1 {
        MirroringMode::Vertical
      } else {
        MirroringMode::Horizontal
      }
    } else {
      mapper_mirroring_mode
    }
  }

  pub fn get_prg_rom(&self) -> Vec<u8> {
    self.prg_rom.clone()
  }

  pub fn get_chr_rom(&self) -> Vec<u8> {
    self.chr_rom.clone()
  }

  pub fn dump_prg_rom(&self) {
    println!("{:?}", self.prg_rom);
  }

  pub fn dump_chr_rom(&self) {
    println!("{:?}", self.chr_rom);
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MirroringMode {
  /// This enum is returned by a mapper if it does not override nametable mirroring
  _Hardwired,
  Horizontal,
  Vertical,
  SingleScreenLow,
  SingleScreenHigh,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Format {
  iNES,
  NES2_0,
  #[default]
  Unknown,
}

#[derive(Clone, Copy, Default)]
pub struct HeaderInfo {
  pub format: Format,
  pub prg_rom_size: u8,
  pub chr_rom_size: u8,
  pub flags6: u8,
  pub flags7: u8,
  pub flags8: u8,
  pub flags9: u8,
  pub flags10: u8,
}

impl Debug for HeaderInfo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("HeaderInfo")
      .field("format", &self.format)
      .field("prg_rom_size", &self.prg_rom_size)
      .field("chr_rom_size", &self.chr_rom_size)
      .field("flags6", &format!("{:08b}", &self.flags6))
      .field("flags7", &format!("{:08b}", &self.flags7))
      .field("flags8", &format!("{:08b}", &self.flags8))
      .field("flags9", &format!("{:08b}", &self.flags9))
      .field("flags10", &format!("{:08b}", &self.flags10))
      .finish()
  }
}

fn parse_header(bytes: &[u8]) -> Result<HeaderInfo, &str> {
  let mut header_info = HeaderInfo::default();

  // Check for NES<EOF> constant, otherwise this is invalid
  if bytes[0] == 0x4E && bytes[1] == 0x45 && bytes[2] == 0x53 && bytes[3] == 0x1A {
    header_info.format = Format::iNES;
  } else {
    return Err("Invalid iNES header");
  }

  // If we've verified that it's iNES-compatible, check for NES2.0 bits
  if header_info.format == Format::iNES && bytes[7] & 0x0C == 0x08 {
    header_info.format = Format::NES2_0;
  }

  header_info.prg_rom_size = bytes[4];
  header_info.chr_rom_size = bytes[5];
  header_info.flags6 = bytes[6];
  header_info.flags7 = bytes[7];
  header_info.flags8 = bytes[8];
  header_info.flags9 = bytes[9];
  header_info.flags10 = bytes[10];

  println!("{:?}", header_info);

  Ok(header_info)
}
