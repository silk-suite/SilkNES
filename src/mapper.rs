pub trait Mapper {
  fn get_mapped_address_cpu(&self, address: u16) -> u32;
  fn get_mapped_address_ppu(&self, address: u16) -> u32;
  //fn mapped_cpu_read(&self, address: u16) -> u32;
  fn mapped_cpu_write(&mut self, address: u16, value: u8);
  //fn mapped_ppu_read(&self, address: u16) -> u32;
  //fn mapped_ppu_write(&self, address: u16, value: u8);
}
