use std::fs;
use std::path::Path;

use crate::mapper::Mapper;
use crate::mapper0::Mapper0;

pub struct Cartridge {
  pub header_info: HeaderInfo,
  pub mapper_id: u8,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  mapper: Box<dyn Mapper>,
}

impl Cartridge {
  pub fn from_rom(rom_path: &str) -> Self {
    let bytes = fs::read(Path::new(rom_path)).expect(&format!("Failed to load ROM from supplied path: {}", rom_path));
    match parse_header(&bytes) {
      Ok(header_info) => {
        let mapper_id = (header_info.flags6 & 0b1111_0000) >> 4 | (header_info.flags7 & 0b1111_0000);
        let mapper = match mapper_id {
          0 => Box::new(Mapper0::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          _ => panic!("Mapper {} not implemented.", mapper_id),
        };
        let prg_start: u16 = 0x0010;
        let prg_end: u16 = prg_start + (0x4000 * header_info.prg_rom_size as u16);
        let chr_start: u16 = prg_end;
        let chr_end: u16 = chr_start + (0x2000 * header_info.chr_rom_size as u16);
        Self {
          header_info,
          mapper_id: 0, // hardcode for now, will detect from ROM in future
          prg_rom: bytes[prg_start as usize..prg_end as usize].to_vec(),
          chr_rom: bytes[chr_start as usize..chr_end as usize].to_vec(),
          mapper,
        }
      },
      Err(_) => panic!("Failed to parse ROM from supplied path: {}.", rom_path),
    }
  }

  pub fn from_bytes(rom_bytes: Vec<u8>) -> Self {
    match parse_header(&rom_bytes) {
      Ok(header_info) => {
        let mapper_id = (header_info.flags6 & 0b1111_0000) >> 4 | (header_info.flags7 & 0b1111_0000);
        let mapper = match mapper_id {
          0 => Box::new(Mapper0::new(header_info.prg_rom_size, header_info.chr_rom_size)) as Box<dyn Mapper>,
          _ => panic!("Mapper {} not implemented.", mapper_id),
        };
        let prg_start: u16 = 0x0010;
        let prg_end: u16 = prg_start + (0x4000 * header_info.prg_rom_size as u16);
        let chr_start: u16 = prg_end;
        let chr_end: u16 = chr_start + (0x2000 * header_info.chr_rom_size as u16);
        Self {
          header_info,
          mapper_id,
          prg_rom: rom_bytes[prg_start as usize..prg_end as usize].to_vec(),
          chr_rom: rom_bytes[chr_start as usize..chr_end as usize].to_vec(),
          mapper,
        }
      },
      Err(_) => panic!("Failed to parse ROM from supplied bytes."),
    }
  }

  pub fn cpu_read(&self, address: u16) -> u8 {
    self.prg_rom[self.mapper.get_mapped_address_cpu(address) as usize]
  }

  pub fn cpu_write(&mut self, address: u16, value: u8) {
    self.prg_rom[self.mapper.get_mapped_address_cpu(address) as usize] = value
  }

  pub fn ppu_read(&self, address: u16) -> u8 {
    let mapped_address = self.mapper.get_mapped_address_ppu(address);
    if (mapped_address as usize) < self.chr_rom.len() {
      self.chr_rom[mapped_address as usize]
    } else {
      panic!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
      //0
    }
  }

  pub fn ppu_write(&mut self, address: u16, value: u8) {
    self.chr_rom[self.mapper.get_mapped_address_ppu(address) as usize] = value
  }

  pub fn get_nametable_layout(&self) -> MirroringMode {
    if self.header_info.flags6 & 0b0000_0001 == 1 {
      MirroringMode::Vertical
    } else {
      MirroringMode::Horizontal
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

pub enum MirroringMode {
  Horizontal,
  Vertical,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Format {
  iNES,
  NES2_0,
  #[default]
  Unknown,
}

#[derive(Clone, Copy, Debug, Default)]
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
