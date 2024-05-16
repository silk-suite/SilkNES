use std::collections::VecDeque;
use std::rc::Rc;
use std::cell::RefCell;

use crate::bus::BusLike;

const LC_LOOKUP: [u8; 32] = [
  10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
  12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30
];

const PULSE_SEQUENCE: [[u8; 8]; 4] = [
  [0, 0, 0, 0, 0, 0, 0, 1],
  [0, 0, 0, 0, 0, 0, 1, 1],
  [0, 0, 0, 0, 1, 1, 1, 1],
  [1, 1, 1, 1, 1, 1, 0, 0],
];

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

impl Pulse {
  pub fn step(&mut self) {
    if self.length_counter > 0 && !self.length_counter_halt {
      self.length_counter = self.length_counter.wrapping_sub(1);
    }
  }

  pub fn get_output(&self) -> f32 {
    0.0
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
  pub fn step(&mut self, half_frame: bool, enabled: bool) {
    if self.linear_counter_reload_flag {
      self.linear_counter = self.linear_counter_reload_value;
    } else if self.linear_counter > 0 {
      self.linear_counter = self.linear_counter.wrapping_sub(1);
    }

    if !self.control_flag {
      self.linear_counter_reload_flag = false;
    }

    if self.length_counter > 0 && half_frame && !self.control_flag && enabled {
      self.length_counter = self.length_counter.wrapping_sub(1);
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
  pub total_cycles: u32,
  pub irq_triggered: bool,
}

impl APU {
  pub fn new() -> Self {
    Self {
      bus: None,
      registers: APURegisters::default(),
      total_cycles: 0,
      irq_triggered: false,
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus);
  }

  pub fn step(&mut self, global_cycles: u32) {
    let mut reset = false;

    if self.registers.triangle.length_counter > 0 && self.registers.triangle.linear_counter > 0 {
      if self.registers.triangle.counter == 0 {
        self.registers.triangle.counter = self.registers.triangle.timer_period;
        self.registers.triangle.sequence_cycle = (self.registers.triangle.sequence_cycle + 1) % 32;
        println!("Incrementing sequence cycle to {}", self.registers.triangle.sequence_cycle);
      }
      self.registers.triangle.counter -= 1;
    }

    if global_cycles % 2 == 0 {
      match self.total_cycles {
        3729 => {
          self.registers.triangle.step(false, self.registers.status.triangle_active);
        }
        7457 => {
          self.registers.triangle.step(true, self.registers.status.triangle_active);
        }
        11186 => {
          self.registers.triangle.step(false, self.registers.status.triangle_active);
        }
        14915 => {
          if self.registers.frame_counter.mode {
            self.registers.triangle.step(true, self.registers.status.triangle_active);
          } else {
            self.registers.triangle.step(false, self.registers.status.triangle_active);
            reset = true;
            if !self.registers.frame_counter.irq_inhibit {
              self.irq_triggered = true;
            }
          }
        },
        18641 => {
          if self.registers.frame_counter.mode {
            self.registers.triangle.step(true, self.registers.status.triangle_active);
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
      0x4015 => self.registers.status.to_u8(),
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
        self.registers.triangle.control_flag = (value & 0b1000_0000) != 0;
        self.registers.triangle.linear_counter = value & 0b0111_1111;
      },
      0x400A => {
        self.registers.triangle.timer_period = (self.registers.triangle.timer_period & 0xFF00) | (value as u16);
      },
      0x400B => {
        self.registers.triangle.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        self.registers.triangle.timer_period = (self.registers.triangle.timer_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8) as u16;
        self.registers.triangle.linear_counter_reload_flag = true;
        println!("Triangle period now: {}", self.registers.triangle.timer_period);
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

  pub fn get_output(&mut self) -> f32 {
    let pulse1_out = self.registers.pulse_1.get_output();
    let pulse2_out = self.registers.pulse_2.get_output();
    let triangle_out = self.registers.triangle.get_output(self.registers.status.triangle_active);
    let noise_out = 0.0;
    let dmc_out = 0.0;

    let pulse_out = 95.88 / ((8218.0 / (pulse1_out + pulse2_out)) + 100.0);
    let tnd_out = 159.79 / ((1.0 / (triangle_out / 8227.0 + noise_out / 12241.0 + dmc_out / 22638.0)) + 100.0);
    2.0 * (pulse_out + tnd_out) - 1.0
  }
}