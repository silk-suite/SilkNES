use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use crate::Cartridge;
use crate::NES6502;
use crate::PPU;

pub trait BusLike {
  fn connect_cpu(&mut self, cpu: Rc<RefCell<NES6502>>);
  fn connect_ppu(&mut self, ppu: Rc<RefCell<PPU>>);
  fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>);
  fn cpu_read(&self, address: u16) -> u8;
  fn cpu_write(&mut self, address: u16, data: u8);
  fn reset(&mut self);
  fn dump_ram(&self) -> Vec<u8>;
  fn get_global_cycles(&self) -> u32;
  fn set_global_cycles(&mut self, cycles: u32);
}

pub struct Bus {
  cpu: Option<Rc<RefCell<NES6502>>>,
  cpu_ram: Vec<u8>,
  ppu: Option<Rc<RefCell<PPU>>>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  pub global_cycles: u32,
}

impl Bus {
  pub fn new() -> Self {
    Self {
      cpu: None,
      cpu_ram: vec![0; 2048],
      ppu: None,
      cartridge: None,
      global_cycles: 0,
    }
  }
}

impl BusLike for Bus {
  fn connect_cpu(&mut self, cpu: Rc<RefCell<NES6502>>) {
    self.cpu = Some(cpu);
  }

  fn connect_ppu(&mut self, ppu: Rc<RefCell<PPU>>) {
    self.ppu = Some(ppu);
  }

  fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
    self.cartridge = Some(Rc::clone(&cartridge));
    if let Some(ppu) = &self.ppu {
      ppu.as_ref().borrow_mut().connect_cartridge(Rc::clone(&cartridge));
    }
  }

  fn cpu_read(&self, address: u16) -> u8 {
    let mut data = 0_u8;

    if address <= 0x1FFF {
      data = self.cpu_ram[(address & 0x07FF) as usize];
    } else if address >= 0x2000 && address <= 0x3FFF {
      if let Some(ppu) = &self.ppu {
        data = ppu.as_ref().borrow_mut().cpu_read(address & 0x0007);
      }
    } else if address >= 0x8000 {
      if let Some(cartridge) = &self.cartridge {
        data = cartridge.as_ref().borrow().cpu_read(address);
      }
    } else {
      data = 0;
    }

    data
  }

  fn cpu_write(&mut self, address: u16, value: u8) {
    if address <= 0x1FFF {
      self.cpu_ram[(address & 0x07FF) as usize] = value;
    } else if address >= 0x2000 && address <= 0x3FFF {
      if let Some(ppu) = &self.ppu {
        ppu.as_ref().borrow_mut().cpu_write(address & 0x0007, value);
      }
    }
  }

  fn reset(&mut self) {
    if let Some(cpu) = self.cpu.borrow() {
      cpu.as_ref().borrow_mut().reset();
    }
  }

  fn dump_ram(&self) -> Vec<u8> {
    println!("{:X?}", self.cpu_ram);
    vec![]
  }

  fn get_global_cycles(&self) -> u32 {
    self.global_cycles
  }

  fn set_global_cycles(&mut self, cycles: u32) {
    self.global_cycles = cycles;
  }
}

pub struct MockBus {
  pub cpu: Option<Rc<RefCell<NES6502>>>,
  pub cpu_ram: Vec<u8>,
}

impl MockBus {
  pub fn new() -> Self {
    Self {
      cpu: None,
      cpu_ram: vec![0; 0x10000],
    }
  }
}

impl BusLike for MockBus {
  fn connect_cpu(&mut self, cpu: Rc<RefCell<NES6502>>) {
    self.cpu = Some(cpu);
  }

  fn connect_ppu(&mut self, ppu: Rc<RefCell<PPU>>) {}

  fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {}

  fn cpu_read(&self, address: u16) -> u8 {
    self.cpu_ram[address as usize]
  }

  fn cpu_write(&mut self, address: u16, value: u8) {
    self.cpu_ram[address as usize] = value;
  }

  fn reset(&mut self) {}

  fn dump_ram(&self) -> Vec<u8> {
    self.cpu_ram.clone()
  }

  fn get_global_cycles(&self) -> u32 {
    0
  }

  fn set_global_cycles(&mut self, cycles: u32) {}
}