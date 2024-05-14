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
  /// Sprite pattern table address for 8x8 sprites
  /// (0: $0000; 1: $1000; ignored in 8x16 mode)
  pub sprite_tile_select: bool,
  /// Background pattern table address (0: $0000; 1: $1000)
  pub background_tile_select: bool,
  /// Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
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
    self.address = (self.address & 0b0111_1100_0001_1111) | ((value as u16) << 5);
  }
  pub fn set_nametable_x(&mut self, value: bool) {
    self.nametable_x = value;
    self.address = (self.address & 0b0111_1011_1111_1111) | ((value as u16) << 10);
  }
  pub fn set_nametable_y(&mut self, value: bool) {
    self.nametable_y = value;
    self.address = (self.address & 0b0111_0111_1111_1111) | ((value as u16) << 11);
  }
  pub fn set_fine_y(&mut self, value: u8) {
    self.fine_y = value;
    self.address = (self.address & 0b0000_1111_1111_1111) | ((value as u16) << 12);
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

pub const COLORS: [[u8; 4]; 0x40] = [
  [98, 98, 98, 255], [0, 31, 178, 255], [36, 4, 200, 255], [82, 0, 178, 255], [115, 0, 118, 255], [128, 0, 36, 255], [115, 11, 0, 255], [82, 40, 0, 255], [36, 68, 0, 255], [0, 87, 0, 255], [0, 92, 0, 255], [0, 83, 36, 255], [0, 60, 118, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 0, 255],
  [171, 171, 171, 255], [13, 87, 255, 255], [75, 48, 255, 255], [138, 19, 255, 255], [188, 8, 214, 255], [210, 18, 105, 255], [199, 46, 0, 255], [157, 84, 0, 255], [96, 123, 0, 255], [32, 152, 0, 255], [0, 163, 0, 255], [0, 153, 66, 255], [0, 125, 180, 255], [0, 0, 0, 255], [0, 0, 0, 255], [0, 0, 0, 255],
  [255, 255, 255, 255], [83, 174, 255, 255], [144, 133, 255, 255], [211, 101, 255, 255], [255, 87, 255, 255], [255, 93, 207, 255], [255, 119, 87, 255], [255, 158, 0, 255], [189, 199, 0, 255], [122, 231, 0, 255], [67, 246, 17, 255], [38, 239, 126, 255], [44, 213, 246, 255], [78, 78, 78, 255], [0, 0, 0, 255], [0, 0, 0, 255],
  [255, 255, 255, 255], [182, 225, 255, 255], [206, 209, 255, 255], [233, 195, 255, 255], [255, 188, 255, 255], [255, 189, 244, 255], [255, 198, 195, 255], [255, 213, 154, 255], [233, 230, 129, 255], [206, 244, 129, 255], [182, 251, 154, 255], [169, 250, 195, 255], [169, 240, 244, 255], [184, 184, 184, 255], [0, 0, 0, 255], [0, 0, 0, 255],
];

#[derive(Debug, Default, Clone, Copy)]
pub struct OAMAttributes {
  pub palette: u8,
  pub priority: bool,
  pub flip_vertically: bool,
  pub flip_horizontally: bool,
}

impl OAMAttributes {
  pub fn to_u8(&self) -> u8 {
    ((self.flip_horizontally as u8) << 7) | ((self.flip_vertically as u8) << 6) | ((self.priority as u8) << 5) | self.palette
  }

  pub fn set_from_u8(&mut self, value: u8) {
    self.palette = value & 0b0000_0011;
    self.priority = (value & 0b0010_0000) > 0;
    self.flip_horizontally = (value & 0b0100_0000) > 0;
    self.flip_vertically = (value & 0b1000_0000) > 0;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OAMSprite {
  pub y: u8,
  pub id: u8,
  pub attributes: OAMAttributes,
  pub x: u8,
}

pub struct PPU {
  bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  screen: [[u8; 4]; 256 * 240],
  pub nametables: [[u8; 0x400]; 2],
  palette: [u8; 32],
  pattern: [[u8; 0x1000]; 2],
  cycle_count: u16,
  scanline_count: i16,
  frame_complete: bool,
  registers: PPURegisters,
  buffered_data: u8,
  pub nmi: bool,
  // Background rendering
  bg_next_tile_id: u8,
  bg_next_tile_attrib: u8,
  bg_next_tile_lsb: u8,
  bg_next_tile_msb: u8,
  bg_pattern_shift_low: u16,
  bg_pattern_shift_high: u16,
  bg_attrib_shift_low: u16,
  bg_attrib_shift_high: u16,
  // Foreground rendering
  pub oam: [OAMSprite; 64],
  oam_address: u8,
  active_sprites: Vec<OAMSprite>,
  sprite_count: u8,
  sprite_shift_low: [u8; 8],
  sprite_shift_high: [u8; 8],
  sprite_zero_hit_possible: bool,
  sprite_zero_being_rendered: bool,
}

impl PPU {
  pub fn new() -> Self {
    Self {
      bus: None,
      cartridge: None,
      screen: [[0, 0, 0, 255]; 256 * 240],
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
      oam: [OAMSprite::default(); 64],
      oam_address: 0,
      active_sprites: Vec::new(),
      sprite_count: 0,
      sprite_shift_low: [0; 8],
      sprite_shift_high: [0; 8],
      sprite_zero_hit_possible: false,
      sprite_zero_being_rendered: false,
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
        let entry = self.oam[(self.oam_address / 4) as usize];
        match self.oam_address % 4 {
          0 => entry.y,
          1 => entry.id,
          2 => entry.attributes.to_u8(),
          3 => entry.x,
          _ => panic!("Invalid OAM address: {:#04X}", self.oam_address),
        }
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
        self.oam_address = value;
      },
      0x0004 => { // OAMDATA
        let entry = &mut self.oam[(self.oam_address / 4) as usize];
        match self.oam_address % 4 {
          0 => entry.y = value,
          1 => entry.id = value,
          2 => entry.attributes.set_from_u8(value),
          3 => entry.x = value,
          _ => panic!("Invalid OAM address: {:#04X}", self.oam_address),
        }
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
      if cartridge.header_info.chr_rom_size > 0 {
        cartridge.ppu_read(address)
      } else {
        self.pattern[((address & 0x1000) >> 12) as usize][(address & 0x0FFF) as usize]
      }
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
      self.palette[pallete_address as usize] & if self.registers.mask.greyscale { 0x30 } else { 0x3F }
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
        // Reset status register values
        self.registers.status.vertical_blank = false;
        self.registers.status.sprite_overflow = false;
        self.registers.status.sprite_zero_hit = false;

        // Reset sprite shifter values
        for i in 0..8 as usize {
          self.sprite_shift_low[i] = 0;
          self.sprite_shift_high[i] = 0;
        }

        // Clear secondary OAM
        self.active_sprites.clear();
      }

      if (self.cycle_count >= 2 && self.cycle_count < 258) || (self.cycle_count >= 321 && self.cycle_count < 338) {
        // Update shifters
        if self.registers.mask.background_enable {
          self.bg_pattern_shift_low <<= 1;
          self.bg_pattern_shift_high <<= 1;
          self.bg_attrib_shift_low <<= 1;
          self.bg_attrib_shift_high <<= 1;
        }

        if self.registers.mask.sprite_enable && self.cycle_count >= 1 && self.cycle_count < 258 {
          for i in 0..self.active_sprites.len() {
            if self.active_sprites[i].x > 0 {
              self.active_sprites[i].x -= 1;
            } else {
              self.sprite_shift_low[i] <<= 1;
              self.sprite_shift_high[i] <<= 1;
            }
          }
        }

        // Run background rendering tasks
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

      if self.cycle_count == 257 && self.scanline_count >= 0 {
        self.active_sprites.clear();
        self.sprite_count = 0;
        for i in 0..8 as usize {
          self.sprite_shift_low[i] = 0;
          self.sprite_shift_high[i] = 0;
        }
        self.sprite_zero_hit_possible = false;

        for i in 0..64 as usize {
          // If diff is positive, scanline is overlapping sprite location
          let diff = self.scanline_count - self.oam[i].y as i16;
          let sprite_size = if self.registers.ctrl.sprite_size { 16 } else { 8 };

          if diff >= 0 && diff < sprite_size {
            if self.sprite_count < 8 {
              if i == 0 {
                self.sprite_zero_hit_possible = true;
              }
              self.active_sprites.push(self.oam[i]);
              self.sprite_count += 1;
            }
          }

          if self.sprite_count == 9 {
            self.registers.status.sprite_overflow = true;
            break;
          }
        }
      }

      if self.cycle_count == 340 {
        for i in 0..self.active_sprites.len() {
          let mut sprite_pattern_bits_low: u8;
          let mut sprite_pattern_bits_high: u8;
          let sprite_pattern_address_low: u16;
          let sprite_pattern_address_high: u16;

          if !self.registers.ctrl.sprite_size { // 8x8 sprites
            if !self.active_sprites[i].attributes.flip_vertically {
              sprite_pattern_address_low = ((self.registers.ctrl.sprite_tile_select as u16) << 12) |
                ((self.active_sprites[i].id as u16) << 4) |
                (self.scanline_count - self.active_sprites[i].y as i16) as u16;
            } else {
              sprite_pattern_address_low = ((self.registers.ctrl.sprite_tile_select as u16) << 12) |
                ((self.active_sprites[i].id as u16) << 4) |
                (7 - (self.scanline_count - self.active_sprites[i].y as i16)) as u16;
            }
          } else { // 8x16 sprites
            if !self.active_sprites[i].attributes.flip_vertically {
              if (self.scanline_count - self.active_sprites[i].y as i16) < 8 {
                // Reading top half of tile
                sprite_pattern_address_low = ((self.active_sprites[i].id as u16 & 0x01) << 12) |
                  ((self.active_sprites[i].id as u16 & 0xFE) << 4) |
                  ((self.scanline_count - self.active_sprites[i].y as i16) & 0x07) as u16;
              } else {
                // Reading bottom half of tile
                sprite_pattern_address_low = ((self.active_sprites[i].id as u16 & 0x01) << 12) |
                  (((self.active_sprites[i].id as u16 & 0xFE) + 1) << 4) |
                  (((self.scanline_count - self.active_sprites[i].y as i16) & 0x07)) as u16;
              }
            } else {
              if (self.scanline_count - self.active_sprites[i].y as i16) < 8 {
                // Reading top half of tile
                sprite_pattern_address_low = ((self.active_sprites[i].id as u16 & 0x01) << 12) |
                  (((self.active_sprites[i].id as u16 & 0xFE) + 1) << 4) |
                  (7 - (self.scanline_count - self.active_sprites[i].y as i16) & 0x07) as u16;
              } else {
                // Reading bottom half of tile
                sprite_pattern_address_low = ((self.active_sprites[i].id as u16 & 0x01) << 12) |
                  (((self.active_sprites[i].id as u16 & 0xFE)) << 4) |
                  (7 - ((self.scanline_count - self.active_sprites[i].y as i16) & 0x07)) as u16;
              }
            }
          }

          sprite_pattern_address_high = sprite_pattern_address_low + 8;

          sprite_pattern_bits_low = self.ppu_read(sprite_pattern_address_low);
          sprite_pattern_bits_high = self.ppu_read(sprite_pattern_address_high);

          if self.active_sprites[i].attributes.flip_horizontally {
            sprite_pattern_bits_low = sprite_pattern_bits_low.reverse_bits();
            sprite_pattern_bits_high = sprite_pattern_bits_high.reverse_bits();
          }

          self.sprite_shift_low[i] = sprite_pattern_bits_low;
          self.sprite_shift_high[i] = sprite_pattern_bits_high;
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

    // Background rendering
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

    // Foreground rendering
    let mut fg_pixel = 0;
    let mut fg_pal = 0;
    let mut fg_priority = 0;
    if self.registers.mask.sprite_enable {
      self.sprite_zero_being_rendered = false;

      for i in 0..self.active_sprites.len() as usize {
        if self.active_sprites[i].x == 0 {
          let fg_pixel_low = ((self.sprite_shift_low[i] & 0x80) > 0) as u8;
          let fg_pixel_high = ((self.sprite_shift_high[i] & 0x80) > 0) as u8;
          fg_pixel = (fg_pixel_high << 1) | fg_pixel_low;

          fg_pal = self.active_sprites[i].attributes.palette + 0x04;
          fg_priority = !(self.active_sprites[i].attributes.priority) as u8;

          if fg_pixel != 0 {
            if i == 0 {
              self.sprite_zero_being_rendered = true;
            }

            break;
          }
        }
      }
    }

    // BG+FG composite
    let mut pixel: u8 = 0;
    let mut pal: u8 = 0;

    if bg_pixel == 0 && fg_pixel == 0 {
      // BG and FG are both transparent, draw background color
      pixel = 0;
      pal = 0;
    } else if bg_pixel == 0 && fg_pixel > 0 {
      // BG is transparent, FG is visible
      pixel = fg_pixel;
      pal = fg_pal;
    } else if bg_pixel > 0 && fg_pixel == 0 {
      // BG is visible, FG is transparent
      pixel = bg_pixel;
      pal = bg_pal;
    } else if bg_pixel > 0 && fg_pixel > 0 {
      // BG and FG are visible, check priority
      if fg_priority > 0 {
        pixel = fg_pixel;
        pal = fg_pal;
      } else {
        pixel = bg_pixel;
        pal = bg_pal;
      }
    }

    if self.sprite_zero_hit_possible && self.sprite_zero_being_rendered {
      if self.registers.mask.background_enable && self.registers.mask.sprite_enable {
        if !(self.registers.mask.background_left_column_enable || self.registers.mask.sprite_left_column_enable) {
          if self.cycle_count >= 9 && self.cycle_count <= 258 {
            self.registers.status.sprite_zero_hit = true;
          }
        } else {
          if self.cycle_count >= 1 && self.cycle_count <= 258 {
            self.registers.status.sprite_zero_hit = true;
          }
        }
      }
    }

    if self.scanline_count < 240 && self.cycle_count < 256 {
      let index = self.scanline_count as usize * 256 + (self.cycle_count as usize - 1);
      if index < self.screen.len() {
        self.screen[index] = self.get_color_from_palette(pal.into(), pixel.into());
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

  pub fn get_color_from_palette(&self, palette: u16, pixel: u16) -> [u8; 4] {
    let index = (self.ppu_read(0x3F00 + (palette << 2) + pixel) & 0x3F) as usize;
    COLORS[index]
  }

  pub fn get_screen(&self) -> Vec<[u8; 4]> {
    Vec::from(self.screen)
  }
}