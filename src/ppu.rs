use crate::bus::BusLike;
use crate::cartridge::{Cartridge, MirroringMode};

use std::rc::Rc;
use std::cell::RefCell;

// region: PPU Registers

#[derive(Debug, Default, Clone, Copy)]
pub struct PPUCTRL {
  pub nametable_x: bool,
  pub nametable_y: bool,
  pub increment_mode: bool,
  pub sprite_tile_select: bool,
  pub background_tile_select: bool,
  pub sprite_size: bool,
  pub slave_mode: bool,
  pub enable_nmi: bool,
}

impl PPUCTRL {
  pub fn to_u8(&self) -> u8 {
    (self.nametable_x as u8) << 0 |
    (self.nametable_y as u8) << 1 |
    (self.increment_mode as u8) << 2 |
    (self.sprite_tile_select as u8) << 3 |
    (self.background_tile_select as u8) << 4 |
    (self.sprite_size as u8) << 5 |
    (self.slave_mode as u8) << 6 |
    (self.enable_nmi as u8) << 7
  }

  pub fn set_from_u8(&mut self, byte: u8) {
    self.nametable_x = (byte & (1 << 0)) != 0;
    self.nametable_y = (byte & (1 << 1)) != 0;
    self.increment_mode = (byte & (1 << 2)) != 0;
    self.sprite_tile_select = (byte & (1 << 3)) != 0;
    self.background_tile_select = (byte & (1 << 4)) != 0;
    self.sprite_size = (byte & (1 << 5)) != 0;
    self.slave_mode = (byte & (1 << 6)) != 0;
    self.enable_nmi = (byte & (1 << 7)) != 0;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PPUMASK {
  pub greyscale: bool,
  pub background_left_column_enable: bool,
  pub sprite_left_column_enable: bool,
  pub background_enable: bool,
  pub sprite_enable: bool,
  pub color_emphasis_red: bool,
  pub color_emphasis_green: bool,
  pub color_emphasis_blue: bool,
}

impl PPUMASK {
  pub fn to_u8(&self) -> u8 {
    (self.greyscale as u8) << 0 |
    (self.background_left_column_enable as u8) << 1 |
    (self.sprite_left_column_enable as u8) << 2 |
    (self.background_enable as u8) << 3 |
    (self.sprite_enable as u8) << 4 |
    (self.color_emphasis_red as u8) << 5 |
    (self.color_emphasis_green as u8) << 6 |
    (self.color_emphasis_blue as u8) << 7
  }

  pub fn set_from_u8(&mut self, byte: u8) {
    self.greyscale = (byte & (1 << 0)) != 0;
    self.background_left_column_enable = (byte & (1 << 1)) != 0;
    self.sprite_left_column_enable = (byte & (1 << 2)) != 0;
    self.background_enable = (byte & (1 << 3)) != 0;
    self.sprite_enable = (byte & (1 << 4)) != 0;
    self.color_emphasis_red = (byte & (1 << 5)) != 0;
    self.color_emphasis_green = (byte & (1 << 6)) != 0;
    self.color_emphasis_blue = (byte & (1 << 7)) != 0;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PPUSTATUS {
  pub sprite_overflow: bool,
  pub sprite_zero_hit: bool,
  pub vertical_blank: bool,
}

impl PPUSTATUS {
  pub fn to_u8(&self) -> u8 {
    (self.sprite_overflow as u8) << 5 |
    (self.sprite_zero_hit as u8) << 6 |
    (self.vertical_blank as u8) << 7
  }

  pub fn set_from_u8(&mut self, byte: u8) {
    self.sprite_overflow = (byte & (1 << 5)) != 0;
    self.sprite_zero_hit = (byte & (1 << 6)) != 0;
    self.vertical_blank = (byte & (1 << 7)) != 0;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Loopy {
  pub coarse_x: u8,
  pub coarse_y: u8,
  pub nametable_x: bool,
  pub nametable_y: bool,
  pub fine_y: u8,
  pub address: u16,
}

impl Loopy {
  pub fn set_coarse_x(&mut self, value: u8) {
    self.coarse_x = value;
    self.address = (self.address & 0b0111_1111_1110_0000) | value as u16;
  }
  pub fn set_coarse_y(&mut self, value: u8) {
    self.coarse_y = value;
    self.address = (self.address & 0b0111_1100_0001_1111) | (value << 5) as u16;
  }
  pub fn set_nametable_x(&mut self, value: bool) {
    self.nametable_x = value;
    self.address = (self.address & 0b0111_1011_1111_1111) | ((value as u16) << 10) as u16;
  }
  pub fn set_nametable_y(&mut self, value: bool) {
    self.nametable_y = value;
    self.address = (self.address & 0b0111_0111_1111_1111) | ((value as u16) << 11) as u16;
  }
  pub fn set_fine_y(&mut self, value: u8) {
    self.fine_y = value;
    self.address = (self.address & 0b0000_1111_1111_1111) | ((value as u16) << 12) as u16;
  }
  pub fn set_address(&mut self, value: u16) {
    self.address = value;
    self.coarse_x = (value & 0b0000_0000_0001_1111) as u8;
    self.coarse_y = ((value & 0b0000_0011_1110_0000) >> 5) as u8;
    self.nametable_x = (value & 0b0000_0100_0000_0000) >> 10 != 0;
    self.nametable_y = (value & 0b0000_1000_0000_0000) >> 11 != 0;
    self.fine_y = ((value & 0b0111_0000_0000_0000) >> 12) as u8;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PPUInternal {
  /// During rendering, used for the scroll position. Outside of rendering, used as the current VRAM address.
  pub v: Loopy,
  /// During rendering, specifies the starting coarse-x scroll for the next scanline
  /// and the starting y scroll for the screen. Outside of rendering, holds the scroll
  /// or VRAM address before transferring it to v.
  pub t: Loopy,
  /// The fine-x position of the current scroll, used during rendering alongside v.
  pub fine_x: u8,
  /// Toggles on each write to either PPUSCROLL or PPUADDR, indicating whether this is the first or second write.
  /// Clears on reads of PPUSTATUS.
  pub write_latch: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PPURegisters {
  pub ctrl: PPUCTRL,
  pub mask: PPUMASK,
  pub status: PPUSTATUS,
  pub oam_address: u8,
  pub oam_data: u8,
  pub scroll: u8,
  pub address: u8,
  pub data: u8,
  pub internal: PPUInternal,
}

// endregion: PPU Registers

pub struct PPU {
  bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  screen: [u8; 256 * 240],
  pub nametables: [[u8; 0x400]; 2],
  palette: [u8; 32],
  pattern: [[u8; 0x1000]; 2],
  cycle_count: u32,
  scanline_count: i32,
  frame_complete: bool,
  registers: PPURegisters,
  buffered_data: u8,
  pub nmi: bool,
  bg_next_tile_id: u8,
  bg_next_tile_attrib: u8,
  bg_next_tile_lsb: u8,
  bg_next_tile_msb: u8,
  bg_pattern_shift_low: u16,
  bg_pattern_shift_high: u16,
  bg_attrib_shift_low: u16,
  bg_attrib_shift_high: u16,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      bus: None,
      cartridge: None,
      screen: [0; 256 * 240],
      nametables: [[0; 0x400]; 2],
      palette: [0; 32],
      pattern: [[0; 0x1000]; 2],
      cycle_count: 0,
      scanline_count: -1,
      frame_complete: false,
      registers: PPURegisters::default(),
      buffered_data: 0,
      nmi: false,
      bg_next_tile_id: 0,
      bg_next_tile_attrib: 0,
      bg_next_tile_lsb: 0,
      bg_next_tile_msb: 0,
      bg_pattern_shift_low: 0,
      bg_pattern_shift_high: 0,
      bg_attrib_shift_low: 0,
      bg_attrib_shift_high: 0,
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus);
  }

  pub fn connect_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
    self.cartridge = Some(cartridge);
  }

  // CPU is reading from PPU
  pub fn cpu_read(&mut self, address: u16) -> u8 {
    match address {
      0x0000 => 0, // CTRL (not readable)
      0x0001 => 0, // MASK (not readable)
      0x0002 => { // STATUS
        // Technically only the top bits of the status register will be used,
        // but we emulate the behavior of the bottom bits being old buffered data
        let data = (self.registers.status.to_u8() & 0xE0) | (self.buffered_data & 0x1F);
        self.registers.status.vertical_blank = false;
        self.registers.internal.write_latch = false;
        data
      },
      0x0003 => 0, // OAMADDR (not readable)
      0x0004 => { // OAMDATA
        println!("TODO: OAMDATA READ");
        0
      },
      0x0005 => 0, // SCROLL (not readable)
      0x0006 => 0, // ADDR (not readable)
      0x0007 => { // DATA
        let mut data = self.buffered_data;
        self.buffered_data = self.ppu_read(self.registers.internal.v.address);

        // Reads from palette memory are not buffered
        if self.registers.internal.v.address >= 0x3F00 {
          data = self.buffered_data;
        }

        let increment = if self.registers.ctrl.increment_mode { 32 } else { 1 };
        self.registers.internal.v.set_address(self.registers.internal.v.address.wrapping_add(increment));

        data
      },
      _ => panic!("Invalid address for PPU read: {:#04X}", address),
    }
  }

  // CPU is writing to PPU
  pub fn cpu_write(&mut self, address: u16, value: u8) {
    match address {
      0x0000 => {
        self.registers.ctrl.set_from_u8(value);
        self.registers.internal.t.set_nametable_x(self.registers.ctrl.nametable_x);
        self.registers.internal.t.set_nametable_y(self.registers.ctrl.nametable_y);
      },
      0x0001 => {
        self.registers.mask.set_from_u8(value);
      },
      0x0002 => {
        panic!("Cannot write to PPU status register");
      },
      0x0003 => { // OAMADDR
        println!("TODO: OAMADDR")
      },
      0x0004 => { // OAMDATA
        println!("TODO: OAMDATA WRITE")
      },
      0x0005 => { // SCROLL
        if !self.registers.internal.write_latch {
          self.registers.internal.fine_x = value & 0x07;
          self.registers.internal.t.set_coarse_x(value >> 3);
          self.registers.internal.write_latch = true;
        } else {
          self.registers.internal.t.set_fine_y(value & 0x07);
          self.registers.internal.t.set_coarse_y(value >> 3);
          self.registers.internal.write_latch = false;
        }
      },
      0x0006 => { // ADDR
        if !self.registers.internal.write_latch {
          self.registers.internal.t.set_address(((value as u16 & 0x3F) << 8) | (self.registers.internal.t.address & 0x00FF));
          self.registers.internal.write_latch = true;
        } else {
          self.registers.internal.t.set_address((self.registers.internal.t.address & 0xFF00) | value as u16);
          self.registers.internal.v.set_address(self.registers.internal.t.address);
          self.registers.internal.write_latch = false;
        }
      },
      0x0007 => { // DATA
        self.ppu_write(self.registers.internal.v.address, value);
        let increment = if self.registers.ctrl.increment_mode { 32 } else { 1 };
        self.registers.internal.v.set_address(self.registers.internal.v.address.wrapping_add(increment));
      },
      _ => panic!("Invalid address for PPU write: {:#04X}", address),
    }
  }

  // PPU is reading from PPU bus
  pub fn ppu_read(&self, address: u16) -> u8 {
    let mut masked = address & 0x3FFF;
    let cartridge = if let Some(cartridge) = &self.cartridge {
      cartridge.borrow()
    } else {
      panic!("Cartridge is not attached to PPU!");
    };
    if masked <= 0x1FFF {
      cartridge.ppu_read(address)
    } else if masked >= 0x2000 && masked <= 0x3EFF {
      //println!("PPU READ from address {:#04X} at scanline {} cycle {}", masked, self.scanline_count, self.cycle_count);
      // Nametables
      masked = address & 0x0FFF;
      match cartridge.get_nametable_layout() {
        MirroringMode::Vertical => {
          match masked {
            0x0000..=0x03FF => self.nametables[0][(masked & 0x03FF) as usize],
            0x0400..=0x07FF => self.nametables[1][(masked & 0x03FF) as usize],
            0x0800..=0x0BFF => self.nametables[0][(masked & 0x03FF) as usize],
            0x0C00..=0x0FFF => self.nametables[1][(masked & 0x03FF) as usize],
            _ => panic!("Invalid address for PPU read: {:#04X}", masked),
          }
        },
        MirroringMode::Horizontal => {
          //println!("Nametable index: {}", ((address & 0x03FF) as usize));
          match masked {
            0x0000..=0x03FF => self.nametables[0][(address & 0x03FF) as usize],
            0x0400..=0x07FF => self.nametables[0][(address & 0x03FF) as usize],
            0x0800..=0x0BFF => self.nametables[1][(address & 0x03FF) as usize],
            0x0C00..=0x0FFF => self.nametables[1][(address & 0x03FF) as usize],
            _ => panic!("Invalid address for PPU read: {:#04X}", address),
          }
        },
      }
    } else if masked >= 0x3F00 && masked <= 0x3FFF {
      let pallete_address = match address & 0x001F {
        0x0010 => self.palette[0x0000 as usize],
        0x0014 => self.palette[0x0004 as usize],
        0x0018 => self.palette[0x0008 as usize],
        0x001C => self.palette[0x000C as usize],
        _ => (address & 0x001F) as u8,
      };
      pallete_address & if self.registers.mask.greyscale { 0x30 } else { 0x3F }
    } else {
      panic!("Invalid address for PPU read: {:#04X}", address);
    }
  }

  // PPU is writing to PPU bus
  pub fn ppu_write(&mut self, address: u16, value: u8) {
    let mut masked = (address & 0x3FFF) as usize;
    let cartridge = if let Some(cartridge) = &self.cartridge {
      cartridge.borrow()
    } else {
      panic!("Cartridge is not attached to PPU!");
    };

    if masked <= 0x1FFF {
      self.pattern[(masked & 0x1000) >> 12][masked & 0x0FFF] = value;
    } else if masked >= 0x2000 && masked <= 0x3EFF {
      masked &= 0x0FFF;
      match cartridge.get_nametable_layout() {
        MirroringMode::Vertical => {
          match masked {
            0x0000..=0x03FF => self.nametables[0][masked & 0x03FF] = value,
            0x0400..=0x07FF => self.nametables[1][masked & 0x03FF] = value,
            0x0800..=0x0BFF => self.nametables[0][masked & 0x03FF] = value,
            0x0C00..=0x0FFF => self.nametables[1][masked & 0x03FF] = value,
            _ => panic!("Invalid address for PPU write: {:#04X}", masked),
          }
        },
        MirroringMode::Horizontal => {
          match masked {
            0x0000..=0x03FF => self.nametables[0][masked & 0x03FF] = value,
            0x0400..=0x07FF => self.nametables[0][masked & 0x03FF] = value,
            0x0800..=0x0BFF => self.nametables[1][masked & 0x03FF] = value,
            0x0C00..=0x0FFF => self.nametables[1][masked & 0x03FF] = value,
            _ => panic!("Invalid address for PPU write: {:#04X}", masked),
          }
        },
      }
    } else if masked >= 0x3F00 && masked <= 0x3FFF {
      let masked = match address & 0x001F {
        0x0010 => 0x0000,
        0x0014 => 0x0004,
        0x0018 => 0x0008,
        0x001C => 0x000C,
        _ => address & 0x001F,
      } as usize;
      self.palette[masked] = value;
    } else {
      panic!("Invalid address for PPU write: {:#04X}", address);
    }
  }

  /// Step the clock of the PPU
  pub fn step(&mut self) {
    if self.scanline_count >= -1 && self.scanline_count < 240 {
      if self.scanline_count == 0 && self.cycle_count == 0 {
        self.cycle_count = 1;
      }

      if self.scanline_count == -1 && self.cycle_count == 1 {
        self.registers.status.vertical_blank = false;
      }

      if (self.cycle_count >= 2 && self.cycle_count < 258) || (self.cycle_count >= 321 && self.cycle_count < 338) {
        // Update shifters
        if self.registers.mask.background_enable {
          self.bg_pattern_shift_low <<= 1;
          self.bg_pattern_shift_high <<= 1;
          self.bg_attrib_shift_low <<= 1;
          self.bg_attrib_shift_high <<= 1;
        }

        match (self.cycle_count - 1) % 8 {
          0 => {
            // Load background shifters
            self.bg_pattern_shift_low = (self.bg_pattern_shift_low & 0xFF00) | self.bg_next_tile_lsb as u16;
            self.bg_pattern_shift_high = (self.bg_pattern_shift_high & 0xFF00) | self.bg_next_tile_msb as u16;

            self.bg_attrib_shift_low = (self.bg_attrib_shift_low & 0xFF00) | if (self.bg_next_tile_attrib & 0b01) != 0 { 0xFF } else { 0 };
            self.bg_attrib_shift_high = (self.bg_attrib_shift_high & 0xFF00) | if (self.bg_next_tile_attrib & 0b10) != 0 { 0xFF } else { 0 };

            self.bg_next_tile_id = self.ppu_read(0x2000 | (self.registers.internal.v.address & 0x0FFF));
          },
          2 => {
            self.bg_next_tile_attrib = self.ppu_read(0x23C0 | ((self.registers.internal.v.nametable_y as u16) << 11)
              | ((self.registers.internal.v.nametable_x as u16) << 10)
              | ((self.registers.internal.v.coarse_y as u16 >> 2) << 3)
              | (self.registers.internal.v.coarse_x as u16 >> 2));

            if (self.registers.internal.v.coarse_y & 0x02) != 0 {
              self.bg_next_tile_attrib >>= 4;
            }
            if (self.registers.internal.v.coarse_x & 0x02) != 0 {
              self.bg_next_tile_attrib >>= 2;
            }

            self.bg_next_tile_attrib &= 0x03;
          },
          4 => {
            self.bg_next_tile_lsb = self.ppu_read(((self.registers.ctrl.background_tile_select as u16) << 12)
              + ((self.bg_next_tile_id as u16) << 4)
              + self.registers.internal.v.fine_y as u16);
          },
          6 => {
            self.bg_next_tile_msb = self.ppu_read(((self.registers.ctrl.background_tile_select as u16) << 12)
              + ((self.bg_next_tile_id as u16) << 4)
              + self.registers.internal.v.fine_y as u16 + 8);
          },
          7 => {
            // Increment scroll X
            if self.registers.mask.background_enable || self.registers.mask.sprite_enable {
              if self.registers.internal.v.coarse_x == 31 {
                self.registers.internal.v.set_coarse_x(0);
                self.registers.internal.v.set_nametable_x(!self.registers.internal.v.nametable_x);
              } else {
                self.registers.internal.v.set_coarse_x(self.registers.internal.v.coarse_x.wrapping_add(1));
              }
            }
          },
          _ => {}
        }
      }

      if self.cycle_count == 256 {
        // Increment scroll Y
        if self.registers.mask.background_enable || self.registers.mask.sprite_enable {
          if self.registers.internal.v.fine_y < 7 {
            self.registers.internal.v.set_fine_y(self.registers.internal.v.fine_y.wrapping_add(1));
          } else {
            self.registers.internal.v.set_fine_y(0);

            if self.registers.internal.v.coarse_y == 29 {
              self.registers.internal.v.set_coarse_y(0);
              self.registers.internal.v.set_nametable_y(!self.registers.internal.v.nametable_y);
            } else if self.registers.internal.v.coarse_y == 31 {
              self.registers.internal.v.set_coarse_y(0);
            } else {
              self.registers.internal.v.set_coarse_y(self.registers.internal.v.coarse_y.wrapping_add(1));
            }
          }
        }
      }

      if self.cycle_count == 257 {
        // Load background shifters
        self.bg_pattern_shift_low = (self.bg_pattern_shift_low & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_pattern_shift_high = (self.bg_pattern_shift_high & 0xFF00) | self.bg_next_tile_msb as u16;

        self.bg_attrib_shift_low = (self.bg_attrib_shift_low & 0xFF00) | if (self.bg_next_tile_attrib & 0b01) != 0 { 0xFF } else { 0 };
        self.bg_attrib_shift_high = (self.bg_attrib_shift_high & 0xFF00) | if (self.bg_next_tile_attrib & 0b10) != 0 { 0xFF } else { 0 };

        // Transfer address X
        if self.registers.mask.background_enable || self.registers.mask.sprite_enable {
          self.registers.internal.v.set_nametable_x(self.registers.internal.t.nametable_x);
          self.registers.internal.v.set_coarse_x(self.registers.internal.t.coarse_x);
        }
      }

      if self.cycle_count == 338 || self.cycle_count == 340 {
        self.bg_next_tile_id = self.ppu_read(0x2000 | (self.registers.internal.v.address & 0x0FFF));
      }

      if self.scanline_count == -1 && self.cycle_count >= 280 && self.cycle_count < 305 {
        // Transfer address Y
        if self.registers.mask.background_enable || self.registers.mask.sprite_enable {
          self.registers.internal.v.set_nametable_y(self.registers.internal.t.nametable_y);
          self.registers.internal.v.set_coarse_y(self.registers.internal.t.coarse_y);
          self.registers.internal.v.set_fine_y(self.registers.internal.t.fine_y);
        }
      }
    }

    if self.scanline_count == 240 {
      // Nothing apparently?
    }

    if self.scanline_count >= 241 && self.scanline_count < 261 {
      if self.scanline_count == 241 && self.cycle_count == 1 {
        self.registers.status.vertical_blank = true;
        if self.registers.ctrl.enable_nmi {
          self.nmi = true;
        }
      }
    }

    let mut bg_pixel = 0;
    let mut bg_pal = 0;
    if self.registers.mask.background_enable {
      let bit_mux = 0x8000 >> self.registers.internal.fine_x;

      let p0_pixel = ((self.bg_pattern_shift_low & bit_mux) > 0) as u8;
      let p1_pixel = ((self.bg_pattern_shift_high & bit_mux) > 0) as u8;
      bg_pixel = (p1_pixel << 1) | p0_pixel;

      let bg_pal0 = ((self.bg_attrib_shift_low & bit_mux) > 0) as u8;
      let bg_pal1 = ((self.bg_attrib_shift_high & bit_mux) > 0) as u8;
      bg_pal = (bg_pal1 << 1) | bg_pal0;

    }

    if self.scanline_count < 240 && self.cycle_count < 256 {
      let index = self.scanline_count as usize * 256 + (self.cycle_count as usize - 1);
      if index < self.screen.len() {
        self.screen[index] = bg_pixel;
      }
    }

    self.cycle_count += 1;
    if self.cycle_count >= 341 {
      self.cycle_count = 0;
      self.scanline_count += 1;
      if self.scanline_count >= 261 {
        self.scanline_count = -1;
        self.frame_complete = true;
      }
    }
  }

  pub fn get_pattern_table(&self, index: u8) -> Vec<u8> {
    let mut vec: Vec<u8> = Vec::new();
    vec.resize(0x4000, 0);

    for tile_y in 0..16 {
      for tile_x in 0..16 {
        let offset: u16 = tile_y * 256 + tile_x * 16;

        for row in 0..8 {
          let mut tile_lsb = self.ppu_read((index as u16 * 0x1000 + offset + row) as u16);
          let mut tile_msb = self.ppu_read((index as u16 * 0x1000 + offset + row + 8) as u16);
          for col in 0..8 {
            let pixel = (tile_lsb & 0x01) + (tile_msb & 0x01);
            tile_lsb >>= 1;
            tile_msb >>= 1;

            let index = ((tile_y * 8 + row) * 256 + (tile_x * 8 + (7 - col))) as usize;
            if index < 0x4000 {
              vec[index] = pixel;
            }
          }
        }
      }
    }

    vec
  }

  pub fn get_palettes(&self) -> Vec<u8> {
    Vec::from(self.palette)
  }

  pub fn get_color_from_palette(&self, palette: u8, pixel: u8) -> u8 {
    self.ppu_read(0x3F00 + (palette << 2) as u16 + pixel as u16)
  }

  pub fn get_screen(&self) -> Vec<u8> {
    Vec::from(self.screen)
  }
}