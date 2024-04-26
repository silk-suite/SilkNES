use crate::bus::Bus;

use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
  pub bus: Option<Rc<RefCell<Bus>>>,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      bus: None,
    }
  }

  // CPU is reading from PPU
  pub fn cpu_read(&self, address: u16) -> u8 {
    let mut data = 0_u8;

    match address {
      0x0000 => todo!(),
      0x0001 => todo!(),
      0x0002 => todo!(),
      0x0003 => todo!(),
      0x0004 => todo!(),
      0x0005 => todo!(),
      0x0006 => todo!(),
      0x0007 => todo!(),
      _ => panic!("Invalid address for PPU read: {:#04x}", address),
    }

    data
  }

  // CPU is writing to PPU
  pub fn cpu_write(&mut self, address: u16, value: u8) {
    let mut data = 0_u8;

    match address {
      0x0000 => todo!(),
      0x0001 => todo!(),
      0x0002 => todo!(),
      0x0003 => todo!(),
      0x0004 => todo!(),
      0x0005 => todo!(),
      0x0006 => todo!(),
      0x0007 => todo!(),
      _ => panic!("Invalid address for PPU write: {:#04x}", address),
    }
  }

  // PPU is reading from PPU bus
  pub fn ppu_read(&self, address: u16) -> u8 {
    let data = 0_u8;
    let addr = address & 0x3FFF;
    data
  }

  // PPU is writing to PPU bus
  pub fn ppu_write(&mut self, address: u16, value: u8) {
    let addr = address & 0x3FFF;
  }
}