use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge;
use crate::Cartridge;
use crate::NES6502;
use crate::PPU;

pub struct Bus {
  cpu: Option<Rc<RefCell<NES6502>>>,
  cpu_ram: Vec<u8>,
  ppu: Option<Rc<RefCell<PPU>>>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl Bus {
  pub fn new() -> Self {
    Self {
      cpu: None,
      cpu_ram: vec![0; 2048],
      ppu: None,
      cartridge: None,
    }
  }

  pub fn connect_cpu(&mut self, cpu: Rc<RefCell<NES6502>>) {
    self.cpu = Some(cpu);
  }

  pub fn connect_ppu(&mut self, ppu: Rc<RefCell<PPU>>) {
    self.ppu = Some(ppu);
  }

  pub fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
    self.cartridge = Some(cartridge);
  }

  pub fn cpu_read(&self, address: u16) -> u8 {
    let mut data = 0_u8;

    if address >= 0x0000 && address <= 0x1FFF {
      data = self.cpu_ram[(address & 0x07FF) as usize];
    } else if address >= 0x2000 && address <= 0x3FFF {
      if let Some(ppu) = &self.ppu {
        data = ppu.borrow().cpu_read(address);
      }
    } else if address >= 0x8000 && address <= 0xFFFF {
      if let Some(cartridge) = &self.cartridge {
        data = cartridge.borrow().mapped_cpu_read(address);
      }
    } else {
      data = 0;
    }

    data
  }

  pub fn cpu_write(&mut self, address: u16, value: u8) {
    if address >= 0x0000 && address <= 0x1FFF {
      self.cpu_ram[(address & 0x07FF) as usize] = value;
    } else if address >= 0x2000 && address <= 0x3FFF {
      if let Some(ppu) = &self.ppu {
        ppu.borrow_mut().cpu_write(address, value);
      }
    }
  }

  pub fn dump_ram(&self) {
    println!("{:X?}", self.cpu_ram);
  }
}