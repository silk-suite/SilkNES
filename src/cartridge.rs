use std::fs;
use std::path::Path;

pub struct Cartridge {
  pub header_info: HeaderInfo,
  pub mapper_id: u8,
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
}

impl Cartridge {
  pub fn from_rom(rom_path: &str) -> Self {
    let bytes = fs::read(Path::new(rom_path)).expect(&format!("Failed to load ROM from supplied path: {}", rom_path));
    match parse_header(&bytes) {
      Ok(header_info) => {
        let prg_start: u16 = 0x0010;
        let prg_end: u16 = prg_start + (0x4000 * header_info.prg_rom_size as u16);
        let chr_start: u16 = prg_end;
        let chr_end: u16 = chr_start + (0x2000 * header_info.chr_rom_size as u16);
        Self {
          header_info,
          mapper_id: 0, // hardcode for now, will detect from ROM in future
          prg_rom: bytes[prg_start as usize..prg_end as usize].to_vec(),
          chr_rom: bytes[chr_start as usize..chr_end as usize].to_vec(),
        }
      },
      Err(_) => panic!("Failed to parse ROM from supplied path: {}.", rom_path),
    }
  }

  pub fn from_bytes(rom_bytes: Vec<u8>) -> Self {
    match parse_header(&rom_bytes) {
      Ok(header_info) => {
        let prg_start: u16 = 0x0010;
        let prg_end: u16 = prg_start + (0x4000 * header_info.prg_rom_size as u16);
        let chr_start: u16 = prg_end;
        let chr_end: u16 = chr_start + (0x2000 * header_info.chr_rom_size as u16);
        Self {
          header_info,
          mapper_id: 0, // hardcode for now, will detect from ROM in future
          prg_rom: rom_bytes[prg_start as usize..prg_end as usize].to_vec(),
          chr_rom: rom_bytes[chr_start as usize..chr_end as usize].to_vec(),
        }
      },
      Err(_) => panic!("Failed to parse ROM from supplied bytes."),
    }
  }

  pub fn mapped_cpu_read(&self, address: u16) -> u8 {
    if self.mapper_id == 0 {
      let address_mask = if self.header_info.prg_rom_size > 1 { 0x7FFF } else { 0x3FFF };
      self.prg_rom[(address & address_mask) as usize]
    } else {
      0
    }
  }
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
