use std::rc::Rc;
use std::cell::RefCell;

use crate::bus::BusLike;

const LC_LOOKUP: [u8; 32] = [
  10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

const PULSE_SEQUENCE: [[f32; 8]; 4] = [
  [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
  [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0],
  [0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
  [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0],
];

#[derive(Debug, Default, Clone, Copy)]
pub struct Pulse {
  duty_cycle: u8,
  length_counter_halt: bool,
  length_counter: u8,
  constant_flag: bool,
  sweep_enabled: bool,
  sweep_period: u8,
  sweep_negate: bool,
  sweep_shift_count: u8,
  sweep_reload_flag: bool,
  sweep_counter: u8,
  timer_period: u16,
  sequencer_cycle: usize,
  sequencer_counter: u16,
  envelope_volume: u8,
  envelope_decay_level: u8,
  envelope_start_flag: bool,
  envelope_counter: u8,
  target_period: u16,
  raw_period: u16,
  muted: bool,
  channel1: bool,
}

impl Pulse {
  pub fn new(channel1: bool) -> Self {
    Pulse {
      channel1,
      ..Default::default()
    }
  }

  pub fn tick_length_counter(&mut self) {
    if self.length_counter > 0 && !self.length_counter_halt {
      self.length_counter -= 1;
    }
  }

  pub fn tick_envelope(&mut self) {
    if !self.envelope_start_flag {
      if self.envelope_counter == 0 {
        self.envelope_counter = self.envelope_volume;
        if self.envelope_decay_level > 0 {
          self.envelope_decay_level -= 1;
        }
        if self.envelope_decay_level == 0 && self.length_counter_halt {
          self.envelope_decay_level = 15;
        }
      } else {
        self.envelope_counter -= 1;
      }
    } else {
      self.envelope_start_flag = false;
      self.envelope_decay_level = 15;
      self.envelope_counter = self.envelope_volume;
    }
  }

  pub fn tick_sweep(&mut self) {
    self.sweep_counter = self.sweep_counter.saturating_sub(1);
    if self.sweep_counter == 0 {
      if self.sweep_shift_count > 0 && self.sweep_enabled && !self.muted {
        self.raw_period = self.target_period;
        self.timer_period = self.raw_period + 1;
        self.update_target_period();
      }

      self.sweep_counter = self.sweep_period;
    }

    if self.sweep_reload_flag {
      self.sweep_reload_flag = false;
      self.sweep_counter = self.sweep_period;
    }

    // Set mute
    self.muted = self.timer_period < 8 || (!self.sweep_negate && self.target_period > 0x07FF);
  }

  pub fn tick_sequencer(&mut self) {
    if self.length_counter > 0 {
      self.sequencer_counter -= 1;
      if self.sequencer_counter == 0 {
        self.sequencer_counter = self.timer_period;
        self.sequencer_cycle = (self.sequencer_cycle + 1) % 8;
      }
    }
  }

  pub fn update_target_period(&mut self) {
    // Calculate target period
    let change_amount = (self.raw_period >> self.sweep_shift_count) as u16;

    if self.sweep_negate {
      self.target_period = self.timer_period.saturating_sub(change_amount);
      if self.channel1 {
        self.target_period -= 1;
      }
    } else {
      self.target_period = self.timer_period.wrapping_add(change_amount);
    }
  }

  pub fn get_output(&mut self, enabled: bool) -> f32 {
    if !enabled || self.length_counter == 0 || self.muted {
      0.0
    } else {
      let duty_cycle_value = PULSE_SEQUENCE[self.duty_cycle as usize][self.sequencer_cycle];
      let envelope_value = if self.constant_flag { self.envelope_volume } else { self.envelope_decay_level };
      duty_cycle_value * envelope_value as f32
    }
  }
}

const TRIANGLE_SEQUENCE: [f32; 32] = [
  15.0, 14.0, 13.0, 12.0, 11.0, 10.0,  9.0,  8.0,  7.0,  6.0,  5.0,  4.0,  3.0,  2.0,  1.0,  0.0,
  0.0,  1.0,  2.0,  3.0,  4.0,  5.0,  6.0,  7.0,  8.0,  9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0
];

#[derive(Debug, Default, Clone, Copy)]
pub struct Triangle {
  control_flag: bool,
  linear_counter_reload_value: u8,
  linear_counter_reload_flag: bool,
  linear_counter: u8,
  length_counter: u8,
  timer_period: u16,
  sequence_cycle: usize,
  counter: u16,
}

impl Triangle {
  pub fn tick_linear_counter(&mut self) {
    if self.linear_counter_reload_flag {
      self.linear_counter = self.linear_counter_reload_value;
    } else if self.linear_counter > 0 {
      self.linear_counter -= 1;
    }

    if !self.control_flag {
      self.linear_counter_reload_flag = false;
    }
  }

  pub fn tick_length_counter(&mut self) {
    if self.length_counter > 0 && !self.control_flag {
      self.length_counter -= 1;
    }
  }

  pub fn tick_sequencer(&mut self) {
    if self.length_counter > 0 && self.linear_counter > 0 {
      self.counter -= 1;
      if self.counter == 0 {
        self.counter = self.timer_period;
        self.sequence_cycle = (self.sequence_cycle + 1) % 32;
      }
    }
  }

  pub fn get_output(&mut self, enabled: bool) -> f32 {
    if !enabled || self.length_counter == 0 || self.linear_counter == 0 {
      0.0
    } else {
      TRIANGLE_SEQUENCE[self.sequence_cycle]
    }
  }
}

const NOISE_PERIOD_SEQUENCE: [u16; 16] = [
  4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
];

#[derive(Debug, Clone, Copy)]
pub struct Noise {
  length_counter_halt: bool,
  constant_flag: bool,
  mode: bool,
  noise_period: u16,
  length_counter: u8,
  shift_register: u16,
  shift_register_timer: u16,
  envelope_volume: u8,
  envelope_decay_level: u8,
  envelope_start_flag: bool,
  envelope_counter: u8,
}

impl Default for Noise {
  fn default() -> Self {
    Self {
      length_counter_halt: false,
      constant_flag: false,
      mode: false,
      noise_period: 0,
      length_counter: 0,
      shift_register: 1,
      shift_register_timer: 0,
      envelope_volume: 0,
      envelope_decay_level: 0,
      envelope_start_flag: false,
      envelope_counter: 0,
    }
  }
}

impl Noise {
  pub fn tick_shift_register(&mut self) {
    if self.shift_register_timer == 0 {
      let feedback = (self.shift_register & 0x1) ^ if self.mode { (self.shift_register & 0x40) >> 6 } else { (self.shift_register & 0x2) >> 1 };
      self.shift_register >>= 1;
      self.shift_register = (self.shift_register & 0x3FFF) | (feedback << 14);
      self.shift_register_timer = self.noise_period;
    }
    self.shift_register_timer = self.shift_register_timer.saturating_sub(1);
  }

  pub fn tick_length_counter(&mut self) {
    if self.length_counter > 0 && !self.length_counter_halt {
      self.length_counter -= 1;
    }
  }

  pub fn tick_envelope(&mut self) {
    if !self.envelope_start_flag {
      self.envelope_counter = self.envelope_counter.saturating_sub(1);
      if self.envelope_counter == 0 {
        self.envelope_counter = self.envelope_volume;
        if self.envelope_decay_level > 0 {
          self.envelope_decay_level -= 1;
        }
        if self.envelope_decay_level == 0 && self.length_counter_halt {
          self.envelope_decay_level = 15;
        }
      }
    } else {
      self.envelope_start_flag = false;
      self.envelope_decay_level = 15;
      self.envelope_counter = self.envelope_volume;
    }
  }

  pub fn get_output(&mut self, enabled: bool) -> f32 {
    if !enabled || self.length_counter == 0 || self.shift_register & 0x1 != 0 {
      0.0
    } else {
      let envelope_value = if self.constant_flag { self.envelope_volume } else { self.envelope_decay_level };
      envelope_value as f32
    }
  }
}

const DMC_RATES: [u16; 16] = [
  428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

#[derive(Clone)]
pub struct DMC {
  irq_enable: bool,
  loop_sample: bool,
  rate: u16,
  output: u8,
  sample_address: u16,
  sample_length: u16,
  // Memory reader
  memory_reader_address: u16,
  bytes_remaining: u16,
  sample_buffer: u8,
  // Ouput unit
  output_unit_timer: u16,
  shift_register: u8,
  bits_remaining: u8,
  silence_flag: bool,
}

impl Default for DMC {
  fn default() -> Self {
    Self {
      irq_enable: false,
      loop_sample: false,
      rate: 0,
      output: 0,
      sample_address: 0xC000,
      sample_length: 1,
      memory_reader_address: 0,
      bytes_remaining: 0,
      sample_buffer: 0,
      output_unit_timer: 0,
      shift_register: 0,
      bits_remaining: 0,
      silence_flag: false,
    }
  }
}

impl DMC {
  pub fn reset(&mut self) {
    self.memory_reader_address = self.sample_address;
    self.bytes_remaining = self.sample_length;
  }

  pub fn tick_output_unit(&mut self) {
    self.output_unit_timer = self.output_unit_timer.saturating_sub(1);
    if self.output_unit_timer == 0 {
      if !self.silence_flag {
        if self.shift_register & 0x1 != 0 && self.output <= 125 {
          self.output += 2;
        } else if self.shift_register & 0x1 == 0 && self.output >= 2 {
          self.output -= 2;
        }
      }
      self.shift_register >>= 1;
      self.bits_remaining = self.bits_remaining.saturating_sub(1);
      if self.bits_remaining == 0 {
        self.bits_remaining = 8;
        if self.sample_buffer == 0 {
          self.silence_flag = true;
        } else {
          self.silence_flag = false;
          self.shift_register = self.sample_buffer;
        }
      }

      self.output_unit_timer = self.rate;
    }
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct APUStatus {
  pub dmc_interrupt: bool,
  pub frame_interrupt: bool,
  dmc_active: bool,
  noise_active: bool,
  triangle_active: bool,
  pulse_2_active: bool,
  pulse_1_active: bool,
}

impl APUStatus {
  pub fn to_u8(&self) -> u8 {
    (self.dmc_interrupt as u8) << 7 |
    (self.frame_interrupt as u8) << 6 |
    (self.dmc_active as u8) << 5 |
    (self.noise_active as u8) << 4 |
    (self.triangle_active as u8) << 3 |
    (self.pulse_2_active as u8) << 2 |
    (self.pulse_1_active as u8) << 1
  }

  pub fn set_from_u8(&mut self, byte: u8) {
    self.dmc_interrupt = (byte & (1 << 7)) != 0;
    self.frame_interrupt = (byte & (1 << 6)) != 0;
    self.dmc_active = (byte & (1 << 5)) != 0;
    self.noise_active = (byte & (1 << 4)) != 0;
    self.triangle_active = (byte & (1 << 3)) != 0;
    self.pulse_2_active = (byte & (1 << 2)) != 0;
    self.pulse_1_active = (byte & (1 << 1)) != 0;
  }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct APUFrameCounter {
  mode: bool,
  irq_inhibit: bool,
}

#[derive(Clone)]
pub struct APURegisters {
  pulse_1: Pulse,
  pulse_2: Pulse,
  triangle: Triangle,
  noise: Noise,
  dmc: DMC,
  pub status: APUStatus,
  frame_counter: APUFrameCounter,
}

impl Default for APURegisters {
  fn default() -> Self {
    Self {
      pulse_1: Pulse::new(true),
      pulse_2: Pulse::new(false),
      triangle: Triangle::default(),
      noise: Noise::default(),
      dmc: DMC::default(),
      status: APUStatus::default(),
      frame_counter: APUFrameCounter::default(),
    }
  }
}

pub struct APU {
  bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
  pub registers: APURegisters,
  pub total_cycles: u32,
  pub irq_pending: bool,
  pub output_buffer: Vec<f32>,
}

impl APU {
  pub fn new() -> Self {
    Self {
      bus: None,
      registers: APURegisters::default(),
      total_cycles: 0,
      irq_pending: false,
      output_buffer: Vec::new(),
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus.clone());
  }

  pub fn read(&self, address: u16) -> u8 {
    if let Some(bus) = &self.bus {
      bus.borrow().cpu_read(address)
    } else {
      panic!("Tried to read from bus before it was connected!");
    }
  }

  pub fn write(&mut self, address: u16, value: u8) {
    if let Some(bus) = &self.bus {
      bus.borrow_mut().cpu_write(address, value);
    } else {
      panic!("Tried to write to bus before it was connected!");
    }
  }

  pub fn tick_quarter_frame(&mut self) {
    self.registers.pulse_1.tick_envelope();
    self.registers.pulse_2.tick_envelope();
    self.registers.noise.tick_envelope();
    self.registers.triangle.tick_linear_counter();
  }

  pub fn tick_half_frame(&mut self) {
    self.registers.pulse_1.tick_envelope();
    self.registers.pulse_2.tick_envelope();
    self.registers.noise.tick_envelope();
    self.registers.triangle.tick_linear_counter();
    self.registers.pulse_1.tick_sweep();
    self.registers.pulse_2.tick_sweep();
    self.registers.pulse_1.tick_length_counter();
    self.registers.pulse_2.tick_length_counter();
    self.registers.triangle.tick_length_counter();
    self.registers.noise.tick_length_counter();
  }

  pub fn step(&mut self, cpu_cycles: u32) {
    let mut reset = false;

    self.registers.pulse_1.update_target_period();
    self.registers.pulse_2.update_target_period();
    self.registers.triangle.tick_sequencer();
    self.registers.noise.tick_shift_register();
    // Don't love doing this here but will fix it later
    // DMC MEMORY READER
    if self.registers.dmc.sample_buffer == 0 && self.registers.dmc.bytes_remaining > 0 {
      self.registers.dmc.sample_buffer = self.read(self.registers.dmc.sample_address);
      self.registers.dmc.memory_reader_address = match self.registers.dmc.memory_reader_address.overflowing_add(1) {
        (_, true) => 0x8000,
        (address, false) => address,
      };
      self.registers.dmc.bytes_remaining -= 1;
      if self.registers.dmc.bytes_remaining == 0 {
        if self.registers.dmc.loop_sample {
          self.registers.dmc.reset();
        } else if self.registers.dmc.irq_enable {
          self.registers.status.dmc_interrupt = true;
        }
      }
    }
    self.registers.dmc.tick_output_unit();

    if cpu_cycles % 2 == 0 {
      self.registers.pulse_1.tick_sequencer();
      self.registers.pulse_2.tick_sequencer();

      match self.total_cycles {
        3729 => {
          self.tick_quarter_frame();
        }
        7457 => {
          self.tick_half_frame();
        }
        11186 => {
          self.tick_quarter_frame();
        }
        14915 => {
          if !self.registers.frame_counter.mode {
            self.tick_half_frame();
            reset = true;
            if !self.registers.frame_counter.irq_inhibit {
              self.registers.status.frame_interrupt = true;
            }
          }
        },
        18641 => {
          if self.registers.frame_counter.mode {
            self.tick_half_frame();
            reset = true;
          }
        }
        _ => {}
      }
  
      self.total_cycles = if reset { 0 } else { self.total_cycles.wrapping_add(1) };
    }
  }

  pub fn cpu_read(&mut self, address: u16) -> u8 {
    match address {
      0x4015 => {
        let mut value = 0;
        if self.registers.pulse_1.length_counter > 0 {
          value |= 0b0000_0001;
        }

        if self.registers.pulse_2.length_counter > 0 {
          value |= 0b0000_0010;
        }

        if self.registers.triangle.length_counter > 0 {
          value |= 0b0000_0100;
        }

        if self.registers.noise.length_counter > 0 {
          value |= 0b0000_1000;
        }

        if self.registers.dmc.bytes_remaining > 0 {
          value |= 0b0001_0000;
        }

        if self.registers.status.frame_interrupt {
          value |= 0b0100_0000;
        }

        if self.registers.status.dmc_interrupt {
          value |= 0b1000_0000;
        }

        self.registers.status.frame_interrupt = false;
        value
      },
      _ => 0
    }
  }

  pub fn cpu_write(&mut self, address: u16, value: u8) {
    //println!("WRITE TO APU at {:#04X}: {:08b}", address, value);
    match address {
      // Pulse 1
      0x4000 => {
        self.registers.pulse_1.duty_cycle = (value & 0b1100_0000) >> 6;
        self.registers.pulse_1.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.pulse_1.constant_flag = value & 0b0001_0000 != 0;
        self.registers.pulse_1.envelope_volume = value & 0b0000_1111;
      },
      0x4001 => {
        self.registers.pulse_1.sweep_enabled = value & 0b1000_0000 != 0;
        self.registers.pulse_1.sweep_period = ((value & 0b0111_0000) >> 4) + 1;
        self.registers.pulse_1.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_1.sweep_shift_count = value & 0b0000_0111;
        self.registers.pulse_1.sweep_reload_flag = true;
        self.registers.pulse_1.update_target_period();
      },
      0x4002 => {
        self.registers.pulse_1.raw_period = (self.registers.pulse_1.raw_period & 0x700) | (value as u16);
        self.registers.pulse_1.timer_period = self.registers.pulse_1.raw_period + 1;
        self.registers.pulse_1.update_target_period();
      },
      0x4003 => {
        if self.registers.status.pulse_1_active {
          self.registers.pulse_1.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        }
        self.registers.pulse_1.raw_period = ((self.registers.pulse_1.raw_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8)) as u16;
        self.registers.pulse_1.timer_period = self.registers.pulse_1.raw_period + 1;
        self.registers.pulse_1.envelope_start_flag = true;
        self.registers.pulse_1.sequencer_cycle = 0;
        self.registers.pulse_1.update_target_period();
      },
      // Pulse 2
      0x4004 => {
        self.registers.pulse_2.duty_cycle = (value & 0b1100_0000) >> 6;
        self.registers.pulse_2.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.pulse_2.constant_flag = value & 0b0001_0000 != 0;
        self.registers.pulse_2.envelope_volume = value & 0b0000_1111;
      },
      0x4005 => {
        self.registers.pulse_2.sweep_enabled = value & 0b1000_0000 != 0;
        self.registers.pulse_2.sweep_period = ((value & 0b0111_0000) >> 4) + 1;
        self.registers.pulse_2.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_2.sweep_shift_count = value & 0b0000_0111;
        self.registers.pulse_2.sweep_reload_flag = true;
        self.registers.pulse_2.update_target_period();
      },
      0x4006 => {
        self.registers.pulse_2.raw_period = (self.registers.pulse_2.raw_period & 0x700) | (value as u16);
        self.registers.pulse_2.timer_period = self.registers.pulse_2.raw_period + 1;
        self.registers.pulse_2.update_target_period();
      },
      0x4007 => {
        if self.registers.status.pulse_2_active {
          self.registers.pulse_2.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        }
        self.registers.pulse_2.raw_period = ((self.registers.pulse_2.raw_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8)) as u16;
        self.registers.pulse_2.timer_period = self.registers.pulse_2.raw_period + 1;
        self.registers.pulse_2.envelope_start_flag = true;
        self.registers.pulse_2.sequencer_cycle = 0;
        self.registers.pulse_2.update_target_period();
      }
      // Triangle
      0x4008 => {
        self.registers.triangle.control_flag = value & 0b1000_0000 != 0;
        self.registers.triangle.linear_counter_reload_value = value & 0b0111_1111;
      },
      0x400A => {
        self.registers.triangle.timer_period = (self.registers.triangle.timer_period & 0xFF00) | (value as u16);
      },
      0x400B => {
        if self.registers.status.triangle_active {
          self.registers.triangle.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        }
        self.registers.triangle.timer_period = (self.registers.triangle.timer_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8) as u16;
        self.registers.triangle.linear_counter_reload_flag = true;
      },
      // Noise
      0x400C => {
        self.registers.noise.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.noise.constant_flag = value & 0b0001_0000 != 0;
        self.registers.noise.envelope_volume = value & 0b0000_1111;
      },
      0x400E => {
        self.registers.noise.mode = value & 0b1000_0000 != 0;
        self.registers.noise.noise_period = NOISE_PERIOD_SEQUENCE[(value & 0b0000_1111) as usize];
      },
      0x400F => {
        if self.registers.status.noise_active {
          self.registers.noise.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        }
        self.registers.noise.envelope_start_flag = true;
      },
      // DMC
      0x4010 => {
        self.registers.dmc.irq_enable = value & 0b1000_0000 != 0;
        self.registers.dmc.loop_sample = value & 0b0100_0000 != 0;
        self.registers.dmc.rate = DMC_RATES[(value & 0b0000_1111) as usize];
      },
      0x4011 => {
        self.registers.dmc.output = value & 0b0111_1111;
      },
      0x4012 => {
        self.registers.dmc.sample_address = 0xC000 + (value * 64) as u16;
      },
      0x4013 => {
        self.registers.dmc.sample_length = (value * 16) as u16 + 1;
      },
      // Status
      0x4015 => {
        self.registers.status.dmc_active = value & 0b0001_0000 != 0;
        if self.registers.status.dmc_active && self.registers.dmc.bits_remaining == 0 {
          self.registers.dmc.reset();
        } else {
          self.registers.dmc.bytes_remaining = 0;
        }
        self.registers.status.noise_active = value & 0b0000_1000 != 0;
        if !self.registers.status.noise_active {
          self.registers.noise.length_counter = 0;
        }
        self.registers.status.triangle_active = value & 0b0000_0100 != 0;
        if !self.registers.status.triangle_active {
          self.registers.triangle.length_counter = 0;
        }
        self.registers.status.pulse_2_active = value & 0b0000_0010 != 0;
        if !self.registers.status.pulse_2_active {
          self.registers.pulse_2.length_counter = 0;
        }
        self.registers.status.pulse_1_active = value & 0b0000_0001 != 0;
        if !self.registers.status.pulse_1_active {
          self.registers.pulse_1.length_counter = 0;
        }

        self.registers.status.dmc_interrupt = false;
      },
      // Frame Counter
      0x4017 => {
        self.registers.frame_counter.mode = value & 0b1000_0000 != 0;
        self.registers.frame_counter.irq_inhibit = value & 0b0100_0000 != 0;
        if self.registers.frame_counter.irq_inhibit {
          self.registers.status.frame_interrupt = false;
          self.irq_pending = true;
        }
        if self.registers.frame_counter.mode {
          self.tick_half_frame();
        }
        self.total_cycles = 0;
      },
      _ => {}
    }
  }

  pub fn update_output(&mut self) {
    // Update output
    let pulse1_out = self.registers.pulse_1.get_output(self.registers.status.pulse_1_active);
    let pulse2_out = self.registers.pulse_2.get_output(self.registers.status.pulse_2_active);
    let triangle_out = self.registers.triangle.get_output(self.registers.status.triangle_active);
    let noise_out = self.registers.noise.get_output(self.registers.status.noise_active);
    let dmc_out = self.registers.dmc.output as f32;

    // // Accurate
    // let pulse_out = 95.88 / ((8218.0 / (pulse1_out + pulse2_out)) + 100.0);
    // let tnd_out = 159.79 / ((1.0 / (triangle_out / 8227.0 + noise_out / 12241.0 + dmc_out / 22638.0)) + 100.0);
    // let output = 2.0 * (pulse_out + tnd_out) - 1.0;

    // Linear Approximate
    let pulse_out = 0.00752 * (pulse1_out + pulse2_out);
    let tnd_out = 0.00851 * triangle_out + 0.00494 * noise_out + 0.00335 * dmc_out;
    let output = 2.0 * (pulse_out + tnd_out) - 1.0;

    self.output_buffer.push(output);
  }
}