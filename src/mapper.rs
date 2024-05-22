use crate::cartridge::MirroringMode;

pub trait Mapper {
  fn get_mapped_address_cpu(&self, address: u16) -> u32;
  fn get_mapped_address_ppu(&self, address: u16) -> u32;
  fn mapped_cpu_write(&mut self, address: u16, value: u8);
  fn mirroring_mode(&self) -> MirroringMode;
}
