use std::rc::Rc;
use std::cell::RefCell;

use crate::bus::BusLike;

pub struct APU {
  bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
}

impl APU {
  pub fn new() -> Self {
    Self {
      bus: None,
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus);
  }
}