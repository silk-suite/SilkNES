use std::collections::VecDeque;
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
  muted: bool,
}

impl Pulse {
  pub fn tick_length_counter(&mut self) {
    if self.length_counter > 0 && !self.length_counter_halt {
      self.length_counter -= 1;
    }
  }

  pub fn tick_envelope(&mut self) {
    if !self.envelope_start_flag {
      self.envelope_counter -= 1;
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

  pub fn tick_sweep(&mut self) {
    if self.sweep_counter == 0 && self.sweep_shift_count > 0 && self.sweep_enabled {
      if !self.muted {
        self.timer_period = self.target_period;
      }

      self.sweep_counter = self.sweep_period;
    }

    self.sweep_counter -= 1;
    if self.sweep_reload_flag {
      self.sweep_reload_flag = false;
      self.sweep_counter = self.sweep_period;
    }

    // Set mute
    self.muted = self.timer_period < 8 || self.target_period > 0x07FF;
  }

  pub fn tick_sequencer(&mut self) {
    if self.length_counter > 0 {
      self.sequencer_counter -= 1;
      if self.sequencer_counter == 0 {
        self.sequencer_counter = self.target_period;
        self.sequencer_cycle = (self.sequencer_cycle + 1) % 8;
      }
    }
  }

  pub fn update_target_period(&mut self) {
    // Calculate target period
    let change_amount = (self.timer_period >> self.sweep_shift_count) as u16;

    if self.sweep_negate {
      self.target_period = self.timer_period.saturating_sub(change_amount);
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

  pub fn step(&mut self, cpu_cycles: u32) {
    let mut reset = false;

    self.registers.triangle.tick_sequencer();

    if cpu_cycles % 2 == 0 {
      self.registers.pulse_1.tick_sequencer();
      self.registers.pulse_2.tick_sequencer();


      match self.total_cycles {
        3729 => {
          self.registers.pulse_1.tick_envelope();
          self.registers.pulse_2.tick_envelope();
          self.registers.triangle.tick_linear_counter();
        }
        7457 => {
          self.registers.pulse_1.tick_envelope();
          self.registers.pulse_2.tick_envelope();
          self.registers.pulse_1.tick_sweep();
          self.registers.pulse_2.tick_sweep();
          self.registers.pulse_1.tick_length_counter();
          self.registers.pulse_2.tick_length_counter();
          self.registers.triangle.tick_linear_counter();
          self.registers.triangle.tick_length_counter();
        }
        11186 => {
          self.registers.pulse_1.tick_envelope();
          self.registers.pulse_2.tick_envelope();
          self.registers.triangle.tick_linear_counter();
        }
        14915 => {
          if !self.registers.frame_counter.mode {
            self.registers.pulse_1.tick_envelope();
            self.registers.pulse_2.tick_envelope();
            self.registers.pulse_1.tick_sweep();
            self.registers.pulse_2.tick_sweep();
            self.registers.pulse_1.tick_length_counter();
            self.registers.pulse_2.tick_length_counter();
            self.registers.triangle.tick_linear_counter();
            self.registers.triangle.tick_length_counter();
            reset = true;
            if !self.registers.frame_counter.irq_inhibit {
              self.irq_triggered = true;
            }
          }
        },
        18641 => {
          if self.registers.frame_counter.mode {
            self.registers.pulse_1.tick_envelope();
            self.registers.pulse_2.tick_envelope();
            self.registers.pulse_1.tick_sweep();
            self.registers.pulse_2.tick_sweep();
            self.registers.pulse_1.tick_length_counter();
            self.registers.pulse_2.tick_length_counter();
            self.registers.triangle.tick_linear_counter();
            self.registers.triangle.tick_length_counter();
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
        self.registers.pulse_1.duty_cycle = value & 0b1100_0000 >> 6;
        self.registers.pulse_1.length_counter_halt = value & 0b0010_0000 != 0;
        self.registers.pulse_1.constant_flag = value & 0b0001_0000 != 0;
        self.registers.pulse_1.envelope_volume = value & 0b0000_1111;
      },
      0x4001 => {
        self.registers.pulse_1.sweep_enabled = value & 0b1000_0000 != 0;
        self.registers.pulse_1.sweep_period = value & 0b0111_0000 >> 4;
        self.registers.pulse_1.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_1.sweep_shift_count = value & 0b0000_0111;
        self.registers.pulse_1.sweep_reload_flag = true;
        self.registers.pulse_1.update_target_period();
      },
      0x4002 => {
        self.registers.pulse_1.timer_period = (self.registers.pulse_1.timer_period & 0xFF00) | (value as u16);
        self.registers.pulse_1.update_target_period();
      },
      0x4003 => {
        self.registers.pulse_1.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        self.registers.pulse_1.timer_period = (self.registers.pulse_1.timer_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8) as u16;
        self.registers.pulse_1.envelope_start_flag = true;
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
        self.registers.pulse_2.sweep_period = value & 0b0111_0000 >> 4;
        self.registers.pulse_2.sweep_negate = value & 0b0000_1000 != 0;
        self.registers.pulse_2.sweep_shift_count = value & 0b0000_0111;
        self.registers.pulse_2.sweep_reload_flag = true;
        self.registers.pulse_2.update_target_period();
      },
      0x4006 => {
        self.registers.pulse_2.timer_period = (self.registers.pulse_2.timer_period & 0xFF00) | (value as u16);
        self.registers.pulse_2.update_target_period();
      },
      0x4007 => {
        self.registers.pulse_2.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        self.registers.pulse_2.timer_period = ((self.registers.pulse_2.timer_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8) as u16);
        self.registers.pulse_2.envelope_start_flag = true;
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
        self.registers.triangle.length_counter = LC_LOOKUP[((value & 0b1111_1000) >> 3) as usize];
        self.registers.triangle.timer_period = (self.registers.triangle.timer_period & 0x00FF) | ((value as u16 & 0b0000_0111) << 8) as u16;
        self.registers.triangle.linear_counter_reload_flag = true;
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
    let pulse1_out = self.registers.pulse_1.get_output(self.registers.status.pulse_1_active);
    let pulse2_out = self.registers.pulse_2.get_output(self.registers.status.pulse_2_active);
    let triangle_out = self.registers.triangle.get_output(self.registers.status.triangle_active);
    let noise_out = 0.0;
    let dmc_out = 0.0;

    let pulse_out = 95.88 / ((8218.0 / (pulse1_out + pulse2_out)) + 100.0);
    let tnd_out = 159.79 / ((1.0 / (triangle_out / 8227.0 + noise_out / 12241.0 + dmc_out / 22638.0)) + 100.0);
    2.0 * (pulse_out + tnd_out) - 1.0
  }
}