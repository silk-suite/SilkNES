use std::rc::Rc;
use std::cell::RefCell;

use crate::bus::BusLike;

#[derive(Debug, Default, Clone, Copy)]
pub struct Pulse {
  duty_cycle: u8,
  length_counter_halt: bool,
  length_counter: u8,
  constant_flag: bool,
  volume: u8,
  sweep_enabled: bool,
  sweep_divider_period: u8,
  sweep_negate: bool,
  sweep_shift_count: u8,
  timer_low: u8,
  timer_high: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Triangle {
  linear_counter_control: bool,
  linear_counter: u8,
  length_counter: u8,
  timer_low: u8,
  timer_high: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Noise {
  length_counter_halt: bool,
  constant_flag: bool,
  volume: u8,
  loop_noise: bool,
  loop_period: u8,
  length_counter: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DMC {
  irq_enable: bool,
  loop_sample: bool,
  frequency_index: u8,
  counter: u8,
  sample_address: u8,
  sample_length: u8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct APUStatus {
  dmc_interrupt: bool,
  frame_interrupt: bool,
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

#[derive(Debug, Default, Clone, Copy)]
pub struct APURegisters {
  pulse_1: Pulse,
  pulse_2: Pulse,
  triangle: Triangle,
  noise: Noise,
  dmc: DMC,
  status: APUStatus,
  frame_counter: APUFrameCounter,
}

pub struct APU {
  bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
  registers: APURegisters,
}

impl APU {
  pub fn new() -> Self {
    Self {
      bus: None,
      registers: APURegisters::default(),
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus);
  }

  pub fn cpu_read(&mut self, address: u16) -> u8 {
    match address {
      0x4015 => self.registers.status.to_u8(),
      _ => 0
    }
  }

  pub fn cpu_write(&mut self, address: u16, value: u8) {
    match address {
      // Pulse 1
      0x4000 => {
        self.registers.pulse_1.duty_cycle = (value & 0b1100_0000) >> 6;
        self.registers.pulse_1.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.pulse_1.constant_flag = (value & 0b0001_0000) != 0;
        self.registers.pulse_1.volume = value & 0b0000_1111;
      },
      0x4001 => {
        self.registers.pulse_1.sweep_enabled = value & 0b1000_0000 != 0;
        self.registers.pulse_1.sweep_divider_period = (value & 0b0111_0000) >> 4;
        self.registers.pulse_1.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_1.sweep_shift_count = value & 0b0000_0111;
      },
      0x4002 => {
        self.registers.pulse_1.timer_low = value;
      },
      0x4003 => {
        self.registers.pulse_1.length_counter = value & 0b1111_1000 >> 3;
        self.registers.pulse_1.timer_high = value & 0b0000_0111;
      },
      // Pulse 2
      0x4004 => {
        self.registers.pulse_2.duty_cycle = (value & 0b1100_0000) >> 6;
        self.registers.pulse_2.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.pulse_2.constant_flag = (value & 0b0001_0000) != 0;
        self.registers.pulse_2.volume = value & 0b0000_1111;
      },
      0x4005 => {
        self.registers.pulse_2.sweep_enabled = value & 0b1000_0000 != 0;
        self.registers.pulse_2.sweep_divider_period = (value & 0b0111_0000) >> 4;
        self.registers.pulse_2.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_2.sweep_shift_count = value & 0b0000_0111;
      },
      0x4006 => {
        self.registers.pulse_2.timer_low = value;
      },
      0x4007 => {
        self.registers.pulse_2.length_counter = value & 0b1111_1000 >> 3;
        self.registers.pulse_2.timer_high = value & 0b0000_0111;
      }
      // Triangle
      0x4008 => {
        self.registers.triangle.linear_counter_control = value & 0b1000_0000 != 0;
        self.registers.triangle.linear_counter = value & 0b0111_1111;
      },
      0x400A => {
        self.registers.triangle.timer_low = value;
      },
      0x400B => {
        self.registers.triangle.length_counter = value & 0b1111_1000 >> 3;
        self.registers.triangle.timer_high = value & 0b0000_0111;
      },
      // Noise
      0x400C => {
        self.registers.noise.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.noise.constant_flag = value & 0b0001_0000 != 0;
        self.registers.noise.volume = value & 0b0000_1111;
      },
      0x400E => {
        self.registers.noise.loop_noise = value & 0b1000_0000 != 0;
        self.registers.noise.loop_period = value & 0b0000_1111;
      },
      0x400F => {
        self.registers.noise.length_counter = value & 0b1111_1000 >> 3;
      },
      // DMC
      0x4010 => {
        self.registers.dmc.irq_enable = value & 0b1000_0000 != 0;
        self.registers.dmc.loop_sample = value & 0b0100_0000 != 0;
        self.registers.dmc.frequency_index = value & 0b0000_1111;
      },
      0x4011 => {
        self.registers.dmc.counter = value & 0b0111_1111;
      },
      0x4012 => {
        self.registers.dmc.sample_address = value;
      },
      0x4013 => {
        self.registers.dmc.sample_length = value;
      },
      // Status
      0x4015 => {
        self.registers.status.dmc_active = value & 0b0001_0000 != 0;
        self.registers.status.noise_active = value & 0b0000_1000 != 0;
        self.registers.status.triangle_active = value & 0b0000_0100 != 0;
        self.registers.status.pulse_2_active = value & 0b0000_0010 != 0;
        self.registers.status.pulse_1_active = value & 0b0000_0001 != 0;
      },
      // Frame Counter
      0x4017 => {
        self.registers.frame_counter.mode = value & 0b1000_0000 != 0;
        self.registers.frame_counter.irq_inhibit = value & 0b0100_0000 != 0;
      },
      _ => {}
    }
  }
}