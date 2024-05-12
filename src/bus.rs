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
  fn update_controller(&mut self, controller_index: usize, value: u8);
}

pub struct Bus {
  cpu: Option<Rc<RefCell<NES6502>>>,
  cpu_ram: Vec<u8>,
  ppu: Option<Rc<RefCell<PPU>>>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  pub controllers: [u8; 2],
  controllers_state: Rc<RefCell<[u8; 2]>>,
  pub global_cycles: u32,
}

impl Bus {
  pub fn new() -> Self {
    Self {
      cpu: None,
      cpu_ram: vec![0; 2048],
      ppu: None,
      cartridge: None,
      controllers: [0, 0],
      controllers_state: Rc::new(RefCell::new([0, 0])),
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
    match address {
      0x0000..=0x1FFF => {
        self.cpu_ram[(address & 0x07FF) as usize]
      },
      0x2000..=0x3FFF => {
        if let Some(ppu) = &self.ppu {
          ppu.as_ref().borrow_mut().cpu_read(address & 0x0007)
        } else {
          panic!("PPU is not connected!");
        }
      },
      0x4016 | 0x4017 => {
        let index = (address & 0x1) as usize;
        let value = (self.controllers_state.as_ref().borrow()[index] & 0x80) > 0;
        self.controllers_state.borrow_mut()[index] <<= 1;
        value as u8
      },
      0x8000..=0xFFFF => {
        if let Some(cartridge) = &self.cartridge {
          cartridge.as_ref().borrow().cpu_read(address)
        } else {
          panic!("Cartridge is not connected!");
        }
      },
      _ => 0
    }
  }

  fn cpu_write(&mut self, address: u16, value: u8) {
    match address {
      0x0000..=0x1FFF => {
        self.cpu_ram[(address & 0x07FF) as usize] = value;
      },
      0x2000..=0x3FFF => {
        if let Some(ppu) = &self.ppu {
          ppu.as_ref().borrow_mut().cpu_write(address & 0x0007, value);
        }
      },
      0x4016 | 0x4017 => {
        let index = (address & 0x1) as usize;
        self.controllers_state.borrow_mut()[index] = self.controllers[index];
      },
      _ => {}
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

  fn update_controller(&mut self, controller_index: usize, value: u8) {
    self.controllers[controller_index] = value;
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

  fn update_controller(&mut self, controller_index: usize, value: u8) {}
}