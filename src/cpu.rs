use crate::bus::BusLike;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddressingMode {
  Implied,
  Immediate,
  ZeroPage,
  ZeroPageX,
  ZeroPageY,
  Relative,
  Absolute,
  AbsoluteX,
  AbsoluteY,
  Indirect,
  IndexedIndirect,
  IndirectIndexed,
}

#[derive(Default)]
pub struct Flags {
  /// The carry flag is set if the last operation caused an overflow
  /// from bit 7 of the result or an underflow from bit 0.
  pub carry: bool,
  /// The zero flag is set if the result of the last operation as was zero.
  pub zero: bool,
  /// The interrupt disable flag is set if the program has executed
  /// a 'Set Interrupt Disable' (SEI) instruction.
  pub interrupt_disable: bool,
  /// While the decimal mode flag is set the processor will obey the rules of
  /// Binary Coded Decimal (BCD) arithmetic during addition and subtraction.
  pub decimal_mode: bool,
  /// The break command bit is set when a BRK instruction has been executed
  /// and an interrupt has been generated to process it.
  pub break_command: bool,
  /// The overflow flag is set during arithmetic operations if the result has yielded an invalid 2's complement result
  /// (e.g. adding to positive numbers and ending up with a negative result: 64 + 64 => -128).
  pub overflow: bool,
  /// The negative flag is set if the result of the last operation had bit 7 set to a one.
  pub negative: bool,
}

impl Flags {
  pub fn to_u8(&self) -> u8 {
    (self.carry as u8) << 0 |
    (self.zero as u8) << 1 |
    (self.interrupt_disable as u8) << 2 |
    (self.decimal_mode as u8) << 3 |
    (self.break_command as u8) << 4 |
    1 << 5 |
    (self.overflow as u8) << 6 |
    (self.negative as u8) << 7
  }

  pub fn from_u8(byte: u8) -> Self {
    Self {
      carry: (byte & (1 << 0)) != 0,
      zero: (byte & (1 << 1)) != 0,
      interrupt_disable: (byte & (1 << 2)) != 0,
      decimal_mode: (byte & (1 << 3)) != 0,
      break_command: (byte & (1 << 4)) != 0,
      overflow: (byte & (1 << 6)) != 0,
      negative: (byte & (1 << 7)) != 0,
    }
  }
}

pub struct NES6502 {
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub sp: u8,
  pub pc: u16,
  pub flags: Flags,
  pub cycles: usize,
  pub bus: Option<Rc<RefCell<Box<dyn BusLike>>>>,
  pub fetched_data: u8,
  pub current_address_abs: u16,
  pub current_address_rel: u16,
  pub total_cycles: u32,
}

impl NES6502 {
  pub fn new() -> Self {
    Self {
      a: 0,
      x: 0,
      y: 0,
      sp: 0xFD,
      pc: 0,
      flags: Default::default(),
      cycles: 0,
      bus: None,
      fetched_data: 0,
      current_address_abs: 0,
      current_address_rel: 0,
      total_cycles: 0,
    }
  }

  pub fn connect_to_bus(&mut self, bus: Rc<RefCell<Box<dyn BusLike>>>) {
    self.bus = Some(bus);
  }

  pub fn step(&mut self) {
    self.total_cycles += 1;
    if self.cycles == 0 {
      //println!("Total cycles: {}", self.total_cycles);
      let opcode = self.read(self.pc);
      println!("PC: {:#04X}, opcode: {:02X}", self.pc, opcode);
      self.pc = self.pc.wrapping_add(1);

      match opcode {
        // ADC
        0x69 => self.adc(AddressingMode::Immediate, 2),
        0x65 => self.adc(AddressingMode::ZeroPage, 3),
        0x75 => self.adc(AddressingMode::ZeroPageX, 4),
        0x6D => self.adc(AddressingMode::Absolute, 4),
        0x7D => self.adc(AddressingMode::AbsoluteX, 4),
        0x79 => self.adc(AddressingMode::AbsoluteY, 4),
        0x61 => self.adc(AddressingMode::IndexedIndirect, 6),
        0x71 => self.adc(AddressingMode::IndirectIndexed, 5),
        // AND
        0x29 => self.and(AddressingMode::Immediate, 2),
        0x25 => self.and(AddressingMode::ZeroPage, 3),
        0x35 => self.and(AddressingMode::ZeroPageX, 4),
        0x2D => self.and(AddressingMode::Absolute, 4),
        0x3D => self.and(AddressingMode::AbsoluteX, 4),
        0x39 => self.and(AddressingMode::AbsoluteY, 4),
        0x21 => self.and(AddressingMode::IndexedIndirect, 6),
        0x31 => self.and(AddressingMode::IndirectIndexed, 5),
        // ASL
        0x0A => self.asl(AddressingMode::Implied, 2),
        0x06 => self.asl(AddressingMode::ZeroPage, 5),
        0x16 => self.asl(AddressingMode::ZeroPageX, 6),
        0x0E => self.asl(AddressingMode::Absolute, 6),
        0x1E => self.asl(AddressingMode::AbsoluteX, 7),
        // BCC
        0x90 => self.bcc(AddressingMode::Relative, 2),
        // BCS
        0xB0 => self.bcs(AddressingMode::Relative, 2),
        // BEQ
        0xF0 => self.beq(AddressingMode::Relative, 2),
        // BIT
        0x24 => self.bit(AddressingMode::ZeroPage, 3),
        0x2C => self.bit(AddressingMode::Absolute, 4),
        // BMI
        0x30 => self.bmi(AddressingMode::Relative, 2),
        // BNE
        0xD0 => self.bne(AddressingMode::Relative, 2),
        // BPL
        0x10 => self.bpl(AddressingMode::Relative, 2),
        // BRK
        0x00 => self.brk(AddressingMode::Implied, 7),
        // BVC
        0x50 => self.bvc(AddressingMode::Relative, 2),
        // BVS
        0x70 => self.bvs(AddressingMode::Relative, 2),
        // CLC
        0x18 => self.clc(AddressingMode::Implied, 2),
        // CLD
        0xD8 => self.cld(AddressingMode::Implied, 2),
        // CLI
        0x58 => self.cli(AddressingMode::Implied, 2),
        // CLV
        0xB8 => self.clv(AddressingMode::Implied, 2),
        // CMP
        0xC9 => self.cmp(AddressingMode::Immediate, 2),
        0xC5 => self.cmp(AddressingMode::ZeroPage, 3),
        0xD5 => self.cmp(AddressingMode::ZeroPageX, 4),
        0xCD => self.cmp(AddressingMode::Absolute, 4),
        0xDD => self.cmp(AddressingMode::AbsoluteX, 4),
        0xD9 => self.cmp(AddressingMode::AbsoluteY, 4),
        0xC1 => self.cmp(AddressingMode::IndexedIndirect, 6),
        0xD1 => self.cmp(AddressingMode::IndirectIndexed, 5),
        // CPX
        0xE0 => self.cpx(AddressingMode::Immediate, 2),
        0xE4 => self.cpx(AddressingMode::ZeroPage, 3),
        0xEC => self.cpx(AddressingMode::Absolute, 4),
        // CPY
        0xC0 => self.cpy(AddressingMode::Immediate, 2),
        0xC4 => self.cpy(AddressingMode::ZeroPage, 3),
        0xCC => self.cpy(AddressingMode::Absolute, 4),
        // DEC
        0xC6 => self.dec(AddressingMode::ZeroPage, 5),
        0xD6 => self.dec(AddressingMode::ZeroPageX, 6),
        0xCE => self.dec(AddressingMode::Absolute, 6),
        0xDE => self.dec(AddressingMode::AbsoluteX, 7),
        // DEX
        0xCA => self.dex(AddressingMode::Implied, 2),
        // DEY
        0x88 => self.dey(AddressingMode::Implied, 2),
        // EOR
        0x49 => self.eor(AddressingMode::Immediate, 2),
        0x45 => self.eor(AddressingMode::ZeroPage, 3),
        0x55 => self.eor(AddressingMode::ZeroPageX, 4),
        0x4D => self.eor(AddressingMode::Absolute, 4),
        0x5D => self.eor(AddressingMode::AbsoluteX, 4),
        0x59 => self.eor(AddressingMode::AbsoluteY, 4),
        0x41 => self.eor(AddressingMode::IndexedIndirect, 6),
        0x51 => self.eor(AddressingMode::IndirectIndexed, 5),
        // INC
        0xE6 => self.inc(AddressingMode::ZeroPage, 5),
        0xF6 => self.inc(AddressingMode::ZeroPageX, 6),
        0xEE => self.inc(AddressingMode::Absolute, 6),
        0xFE => self.inc(AddressingMode::AbsoluteX, 7),
        // INX
        0xE8 => self.inx(AddressingMode::Implied, 2),
        // INY
        0xC8 => self.iny(AddressingMode::Implied, 2),
        // JMP
        0x4C => self.jmp(AddressingMode::Absolute, 3),
        0x6C => self.jmp(AddressingMode::Indirect, 5),
        // JSR
        0x20 => self.jsr(AddressingMode::Absolute, 6),
        // LDA
        0xA9 => self.lda(AddressingMode::Immediate, 2),
        0xA5 => self.lda(AddressingMode::ZeroPage, 3),
        0xB5 => self.lda(AddressingMode::ZeroPageX, 4),
        0xAD => self.lda(AddressingMode::Absolute, 4),
        0xBD => self.lda(AddressingMode::AbsoluteX, 4),
        0xB9 => self.lda(AddressingMode::AbsoluteY, 4),
        0xA1 => self.lda(AddressingMode::IndexedIndirect, 6),
        0xB1 => self.lda(AddressingMode::IndirectIndexed, 5),
        // LDX
        0xA2 => self.ldx(AddressingMode::Immediate, 2),
        0xA6 => self.ldx(AddressingMode::ZeroPage, 3),
        0xB6 => self.ldx(AddressingMode::ZeroPageY, 4),
        0xAE => self.ldx(AddressingMode::Absolute, 4),
        0xBE => self.ldx(AddressingMode::AbsoluteY, 4),
        // LDY
        0xA0 => self.ldy(AddressingMode::Immediate, 2),
        0xA4 => self.ldy(AddressingMode::ZeroPage, 3),
        0xB4 => self.ldy(AddressingMode::ZeroPageX, 4),
        0xAC => self.ldy(AddressingMode::Absolute, 4),
        0xBC => self.ldy(AddressingMode::AbsoluteX, 4),
        // LSR
        0x4A => self.lsr(AddressingMode::Implied, 2),
        0x46 => self.lsr(AddressingMode::ZeroPage, 5),
        0x56 => self.lsr(AddressingMode::ZeroPageX, 6),
        0x4E => self.lsr(AddressingMode::Absolute, 6),
        0x5E => self.lsr(AddressingMode::AbsoluteX, 7),
        // NOP
        0xEA => self.nop(AddressingMode::Implied, 2),
        // ORA
        0x09 => self.ora(AddressingMode::Immediate, 2),
        0x05 => self.ora(AddressingMode::ZeroPage, 3),
        0x15 => self.ora(AddressingMode::ZeroPageX, 4),
        0x0D => self.ora(AddressingMode::Absolute, 4),
        0x1D => self.ora(AddressingMode::AbsoluteX, 4),
        0x19 => self.ora(AddressingMode::AbsoluteY, 4),
        0x01 => self.ora(AddressingMode::IndexedIndirect, 6),
        0x11 => self.ora(AddressingMode::IndirectIndexed, 5),
        // PHA
        0x48 => self.pha(AddressingMode::Implied, 3),
        // PHP
        0x08 => self.php(AddressingMode::Implied, 3),
        // PLA
        0x68 => self.pla(AddressingMode::Implied, 4),
        // PLP
        0x28 => self.plp(AddressingMode::Implied, 4),
        // ROL
        0x2A => self.rol(AddressingMode::Implied, 2),
        0x26 => self.rol(AddressingMode::ZeroPage, 5),
        0x36 => self.rol(AddressingMode::ZeroPageX, 6),
        0x2E => self.rol(AddressingMode::Absolute, 6),
        0x3E => self.rol(AddressingMode::AbsoluteX, 7),
        // ROR
        0x6A => self.ror(AddressingMode::Implied, 2),
        0x66 => self.ror(AddressingMode::ZeroPage, 5),
        0x76 => self.ror(AddressingMode::ZeroPageX, 6),
        0x6E => self.ror(AddressingMode::Absolute, 6),
        0x7E => self.ror(AddressingMode::AbsoluteX, 7),
        // RTI
        0x40 => self.rti(AddressingMode::Implied, 6),
        // RTS
        0x60 => self.rts(AddressingMode::Implied, 6),
        // SBC
        0xE9 => self.sbc(AddressingMode::Immediate, 2),
        0xE5 => self.sbc(AddressingMode::ZeroPage, 3),
        0xF5 => self.sbc(AddressingMode::ZeroPageX, 4),
        0xED => self.sbc(AddressingMode::Absolute, 4),
        0xFD => self.sbc(AddressingMode::AbsoluteX, 4),
        0xF9 => self.sbc(AddressingMode::AbsoluteY, 4),
        0xE1 => self.sbc(AddressingMode::IndexedIndirect, 6),
        0xF1 => self.sbc(AddressingMode::IndirectIndexed, 5),
        // SEC
        0x38 => self.sec(AddressingMode::Implied, 2),
        // SED
        0xF8 => self.sed(AddressingMode::Implied, 2),
        // SEI
        0x78 => self.sei(AddressingMode::Implied, 2),
        // STA
        0x85 => self.sta(AddressingMode::ZeroPage, 3),
        0x95 => self.sta(AddressingMode::ZeroPageX, 4),
        0x8D => self.sta(AddressingMode::Absolute, 4),
        0x9D => self.sta(AddressingMode::AbsoluteX, 5),
        0x99 => self.sta(AddressingMode::AbsoluteY, 5),
        0x81 => self.sta(AddressingMode::IndexedIndirect, 6),
        0x91 => self.sta(AddressingMode::IndirectIndexed, 6),
        // STX
        0x86 => self.stx(AddressingMode::ZeroPage, 3),
        0x96 => self.stx(AddressingMode::ZeroPageY, 4),
        0x8E => self.stx(AddressingMode::Absolute, 4),
        // STY
        0x84 => self.sty(AddressingMode::ZeroPage, 3),
        0x94 => self.sty(AddressingMode::ZeroPageX, 4),
        0x8C => self.sty(AddressingMode::Absolute, 4),
        // TAX
        0xAA => self.tax(AddressingMode::Implied, 2),
        // TAY
        0xA8 => self.tay(AddressingMode::Implied, 2),
        // TSX
        0xBA => self.tsx(AddressingMode::Implied, 2),
        // TXA
        0x8A => self.txa(AddressingMode::Implied, 2),
        // TXS
        0x9A => self.txs(AddressingMode::Implied, 2),
        // TYA
        0x98 => self.tya(AddressingMode::Implied, 2),
        // Any other opcode gets caught here
        _ => {
          println!("Invalid opcode: {}", opcode);
          self.cycles = 1;
        },
      }
    }

    self.cycles -= 1;
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

  fn fetch(&mut self, mode: AddressingMode) {
    match mode {
      // Data has an implicit source, potentially the accumulator
      AddressingMode::Implied => {
        self.fetched_data = self.a;
      },
      // The data is immediately available in the following byte
      AddressingMode::Immediate => {
        self.current_address_abs = self.pc;
        self.pc = self.pc.wrapping_add(1);
      },
      // Addressing 0x0000 to 0x00FF only
      AddressingMode::ZeroPage => {
        self.current_address_abs = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.current_address_abs &= 0x00FF;
      },
      // Index into the zero page with X offset
      AddressingMode::ZeroPageX => {
        self.current_address_abs = (self.read(self.pc).wrapping_add(self.x)) as u16 % 0xFFFF;
        self.pc = self.pc.wrapping_add(1);
        self.current_address_abs &= 0x00FF;
      },
      // Index into the zero page with Y offset
      AddressingMode::ZeroPageY => {
        self.current_address_abs = (self.read(self.pc) + self.y) as u16;
        self.pc = self.pc.wrapping_add(1);
        self.current_address_abs &= 0x00FF;
      },
      AddressingMode::Relative => {
        self.current_address_rel = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        // Check if relative address is negative
        if self.current_address_rel & 0x80 != 0 {
          self.current_address_rel |= 0xFF00
        }
      },
      // Read the next two bytes as a 16-bit address
      AddressingMode::Absolute => {
        let low = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let high = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.current_address_abs = (high << 8) | low;
      },
      // Read the next two bytes as a 16-bit address, and add X offset
      AddressingMode::AbsoluteX => {
        let low = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let high = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.current_address_abs = (high << 8) | low;
        self.current_address_abs = self.current_address_abs.wrapping_add(self.x as u16);

        if (self.current_address_abs & 0xFF00) != (high << 8) {
          // Crossed page boundary, add an additional clock cycle
          self.cycles += 1;
        }
      },
      // Read the next two bytes as a 16-bit address, and add Y offset
      AddressingMode::AbsoluteY => {
        let low = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let high = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        self.current_address_abs = (high << 8) | low;
        self.current_address_abs = self.current_address_abs.wrapping_add(self.y as u16);

        if (self.current_address_abs & 0xFF00) != (high << 8) {
          // Crossed page boundary, add an additional clock cycle
          self.cycles += 1;
        }
      },
      AddressingMode::Indirect => {
        let ptr_low = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let ptr_high = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let ptr = (ptr_high << 8) | ptr_low;

        if ptr_low == 0x00FF {
          // Simulates hardware page boundary bug
          self.current_address_abs = (self.read(ptr & 0xFF00) as u16) << 8 | self.read(ptr) as u16;
        } else {
          self.current_address_abs = (((self.read(ptr + 1) as u16) << 8) | self.read(ptr) as u16) as u16;
        }
      },
      // Index into address table on the zero page and offset by X
      // val = PEEK(PEEK((arg + X) % 256) + PEEK((arg + X + 1) % 256) * 256)
      AddressingMode::IndexedIndirect => {
        let operand = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let low = self.read((operand.wrapping_add(self.x as u16)) & 0xFF) as u16;
        let high = self.read((operand.wrapping_add(self.x as u16 + 1)) & 0xFF) as u16;

        self.current_address_abs = (high << 8) | low;
      },
      // Index into the zero page, read 16-bit address, and add Y offset to it
      // val = PEEK(PEEK(arg) + PEEK((arg + 1) % 256) * 256 + Y)
      AddressingMode::IndirectIndexed => {
        let table = self.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);

        let low = self.read((table as u16) & 0x00FF) as u16;
        let high = self.read((table.wrapping_add(1)) as u16 & 0x00FF) as u16;

        self.current_address_abs = (high << 8) | low;
        self.current_address_abs = self.current_address_abs.wrapping_add(self.y as u16);

        if (self.current_address_abs & 0xFF00) != (high << 8) {
          // Crossed page boundary, add an additional clock cycle
          self.cycles += 1;
        }
      },
    }

    if mode != AddressingMode::Implied {
      self.fetched_data = self.read(self.current_address_abs);
    }
  }

  // region: Instructions

  /// Add with carry
  fn adc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let temp = self.a as u16 + self.fetched_data as u16 + self.flags.carry as u16;
    self.flags.carry = temp > 255;
    self.flags.zero = (temp & 0x00FF) == 0;
    self.flags.negative = temp & 0x80 != 0;
    self.flags.overflow = (!(self.a as u16 ^ self.fetched_data as u16) & (self.a as u16 ^ temp)) & 0x0080 != 0;

    self.a = (temp & 0x00FF) as u8;
  }

  /// Logical AND accumulator with given data
  fn and(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    println!("Fetched data: {}", self.fetched_data);

    self.a &= self.fetched_data;

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  /// Arithmetic shift left
  fn asl(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let value = (self.fetched_data as u16) << 1;

    self.flags.carry = value & 0xFF00 != 0;
    self.flags.zero = value & 0x00FF == 0;
    self.flags.negative = value & 0x80 != 0;

    if mode == AddressingMode::Implied {
      self.a = (value & 0x00FF) as u8;
    } else {
      self.write(self.current_address_abs, (value & 0x00FF) as u8);
    }
  }

  /// Branch if carry flag is clear
  fn bcc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if !self.flags.carry {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Branch if carry flag is set
  fn bcs(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if self.flags.carry {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Branch if zero flag is set
  fn beq(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if self.flags.zero {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// AND the contents of A with the value in memory and check if bits are set
  fn bit(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let temp = self.a & self.fetched_data;

    self.flags.zero = (temp & 0x00FF) == 0;
    self.flags.overflow = temp & 0x40 != 0;
    self.flags.negative = temp & 0x80 != 0;
  }

  /// Branch if negative flag is set
  fn bmi(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if self.flags.negative {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Branch if zero flag is clear
  fn bne(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if !self.flags.zero {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Branch if negative flag is clear
  fn bpl(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if !self.flags.negative {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Forces the generation of an interrupt request
  fn brk(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.pc += 1;

    // Push the program counter onto the stack
    self.flags.interrupt_disable = true;
    self.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8 & 0x00FF);
    self.sp -= 1;
    self.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);

    self.flags.break_command = true;
    self.write(0x0100 + self.sp as u16, self.flags.to_u8());
    self.sp -= 1;
    self.flags.break_command = false;

    self.pc = self.read(0xFFFE) as u16 | ((self.read(0xFFFF) as u16) << 8) as u16;
  }

  /// Branch if overflow flag is clear
  fn bvc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if !self.flags.overflow {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Branch if overflow flag is set
  fn bvs(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    if self.flags.overflow {
      self.cycles += 1;
      self.current_address_abs = self.pc.wrapping_add(self.current_address_rel);

      if (self.current_address_abs & 0xFF00) != (self.pc & 0xFF00) {
        // Crossed page boundary, add an additional clock cycle
        self.cycles += 1;
      }

      self.pc = self.current_address_abs;
    }
  }

  /// Clear carry flag
  fn clc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.carry = false;
  }

  /// Clear decimal mode
  fn cld(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.decimal_mode = false;
  }

  /// Clear interrupt disable flag
  fn cli(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.interrupt_disable = false;
  }

  /// Clear overflow flag
  fn clv(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.overflow = false;
  }

  /// Compare the contents of the accumulator with another value in memory
  fn cmp(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.carry = self.a >= self.fetched_data;
    self.flags.zero = ((self.a - self.fetched_data) & 0x00FF) == 0;
    self.flags.negative = (self.a - self.fetched_data) & 0x80 != 0;
  }

  /// Compare the contents of the X register with another value in memory
  fn cpx(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.carry = self.x >= self.fetched_data;
    self.flags.zero = ((self.x - self.fetched_data) & 0x00FF) == 0;
    self.flags.negative = (self.x - self.fetched_data) & 0x80 != 0;
  }

  /// Compare the contents of the Y register with another value in memory
  fn cpy(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.carry = self.y >= self.fetched_data;
    self.flags.zero = ((self.y - self.fetched_data) & 0x00FF) == 0;
    self.flags.negative = (self.y - self.fetched_data) & 0x80 != 0;
  }

  /// Decrement value stored at memory address by 1
  fn dec(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    // Make this better later
    let mut value = self.read(self.current_address_abs);
    self.write(self.current_address_abs, value.wrapping_sub(1) & 0x00FF);
    value = self.read(self.current_address_abs);

    self.flags.zero = (value & 0x00FF) == 0;
    self.flags.negative = (value & 0x80) != 0;
  }

  /// Decrement X register by 1
  fn dex(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.x -= 1;

    self.flags.zero = self.x == 0;
    self.flags.negative = (self.x & 0x80) != 0;
  }

  /// Decrement Y register by 1
  fn dey(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.y -= 1;

    self.flags.zero = self.y == 0;
    self.flags.negative = (self.y & 0x80) != 0;
  }

  /// Logical XOR accummulator with given value
  fn eor(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.a ^= self.fetched_data;

    self.flags.zero = self.a == 0;
    self.flags.negative = (self.a & 0x80) != 0;
  }

  /// Increment value stored at memory address by 1
  fn inc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    // Make this better later
    let mut value = self.read(self.current_address_abs);
    self.write(self.current_address_abs, value.wrapping_add(1));
    value = self.read(self.current_address_abs);

    self.flags.zero = value == 0;
    self.flags.negative = (value & 0x80) != 0;
  }

  /// Increment X register by 1
  fn inx(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.x = self.x.wrapping_add(1);

    self.flags.zero = self.x == 0;
    self.flags.negative = (self.x & 0x80) != 0;
  }

  /// Increment Y register by 1
  fn iny(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.y = self.y.wrapping_add(1);

    self.flags.zero = self.y == 0;
    self.flags.negative = (self.y & 0x80) != 0;
  }

  /// Set the program counter to the given address
  fn jmp(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.pc = self.current_address_abs;
  }

  // Push the current program counter to the stack, then jump to the given address
  fn jsr(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.pc -= 1;

    self.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8 & 0x00FF);
    self.sp -= 1;
    self.write(0x0100 + self.sp as u16, self.pc as u8 & 0x00FF);
    self.sp -= 1;

    self.pc = self.current_address_abs;
  }

  /// Load a byte of memory into the accumulator
  fn lda(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.a = self.fetched_data;

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  /// Load a byte of memory into the X register
  fn ldx(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.x = self.fetched_data;

    self.flags.zero = self.x == 0;
    self.flags.negative = self.x & 0x80 != 0;
  }

  /// Load a byte of memory into the Y register
  fn ldy(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.y = self.fetched_data;

    self.flags.zero = self.y == 0;
    self.flags.negative = self.y & 0x80 != 0;
  }

  /// Logical shift right
  fn lsr(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let original_value = self.fetched_data as u16;
    let value = (original_value >> 1) as u8;

    self.flags.carry = (original_value & 0x01) != 0;
    self.flags.zero = (value & 0x00FF) == 0;
    self.flags.negative = (value & 0x80) != 0;

    if mode == AddressingMode::Implied {
      self.a = (value & 0x00FF) as u8;
    } else {
      self.write(self.current_address_abs, (value & 0x00FF) as u8);
    }
  }

  /// No op
  fn nop(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);
  }

  /// Logical OR the accumulator with a byte of memory
  fn ora(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.a |= self.fetched_data;

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  /// Pushes a copy of the accumulator on to the stack.
  fn pha(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.write(0x0100 + self.sp as u16, self.a);
    self.sp -= 1;
  }

  /// Pushes a copy of the status flags on to the stack.
  fn php(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.write(0x0100 + self.sp as u16, self.flags.to_u8());
    self.sp -= 1;
  }

  /// Pulls an 8 bit value from the stack and into the accumulator.
  fn pla(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.sp += 1;
    self.a = self.read(0x0100 + self.sp as u16);

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  /// Pulls an 8 bit value from the stack and into the processor flags.
  fn plp(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.sp += 1;
    self.flags = Flags::from_u8(self.read(0x0100 + self.sp as u16));
  }

  /// Move each of the bits in either A or M one place to the left.
  fn rol(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let value = ((self.fetched_data << 1) | self.flags.carry as u8) as u16;

    self.flags.carry = value & 0xFF00 != 0;
    self.flags.zero = value == 0;
    self.flags.negative = (value & 0x80) != 0;

    if mode == AddressingMode::Implied {
      self.a = (value & 0x00FF) as u8;
    } else {
      self.write(self.current_address_abs, (value & 0x00FF) as u8);
    }
  }

  /// Move each of the bits in either A or M one place to the right.
  fn ror(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let value = ((self.flags.carry as u8) << 7) as u16 | (self.fetched_data >> 1) as u16;

    self.flags.carry = value & 0x01 != 0;
    self.flags.zero = (value & 0x00FF) == 0;
    self.flags.negative = (value & 0x80) != 0;

    if mode == AddressingMode::Implied {
      self.a = (value & 0x00FF) as u8;
    } else {
      self.write(self.current_address_abs, (value & 0x00FF) as u8);
    }
  }

  /// Return from interrupt
  fn rti(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    // Pull status flags
    self.sp += 1;
    self.flags = Flags::from_u8(self.read(0x0100 + self.sp as u16));
    self.flags.break_command = !self.flags.break_command;

    // Pull program counter
    self.sp += 1;
    self.pc = self.read(0x0100 + self.sp as u16) as u16;
    self.sp += 1;
    self.pc |= (self.read(0x0100 + self.sp as u16) as u16) << 8;
  }

  /// Pull the program counter from the stack (minus one) and jump to it
  fn rts(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.sp += 1;
    self.pc = self.read(0x0100 + self.sp as u16) as u16;
    self.sp += 1;
    self.pc |= (self.read(0x0100 + self.sp as u16) as u16) << 8;

    self.pc += 1;
  }

  /// Subtraction with carry
  fn sbc(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    let value = self.fetched_data as u16 ^ 0x00FF;
    let temp = self.a as u16 + value + self.flags.carry as u16;
    self.flags.carry = temp & 0xFF00 != 0;
    self.flags.zero = (temp & 0x00FF) == 0;
    self.flags.negative = temp & 0x80 != 0;
    self.flags.overflow = (((temp ^ self.a as u16) & (temp ^ value)) & 0x0080) != 0;

    self.a = (temp & 0x00FF) as u8;
  }

  /// Set carry
  fn sec(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.carry = true;
  }

  /// Set decimal mode
  fn sed(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.decimal_mode = true;
  }

  /// Set the interrupt disable flag
  fn sei(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.flags.interrupt_disable = true;
  }

  /// Store the contents of A in memory
  fn sta(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.write(self.current_address_abs, self.a);
  }

  /// Store the contents of register X in memory
  fn stx(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.write(self.current_address_abs, self.x);
  }

  /// Store the contents of register Y in memory
  fn sty(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.write(self.current_address_abs, self.y);
  }

  /// Transfer the contents of A to register X
  fn tax(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.x = self.a;

    self.flags.zero = self.x == 0;
    self.flags.negative = self.x & 0x80 != 0;
  }

  /// Transfer the contents of A to register Y
  fn tay(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.y = self.a;

    self.flags.zero = self.y == 0;
    self.flags.negative = self.y & 0x80 != 0;
  }

  /// Transfer the contents of the stack register to register X
  fn tsx(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.x = self.sp;

    self.flags.zero = self.x == 0;
    self.flags.negative = self.x & 0x80 != 0;
  }

  /// Transfer the contents of register X to A
  fn txa(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.a = self.x;

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  /// Transfer the contents of register X to the stack register
  fn txs(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.sp = self.x;
  }

  /// Transfer the contents of register Y to A
  fn tya(&mut self, mode: AddressingMode, initial_cycle_count: usize) {
    self.cycles += initial_cycle_count;
    self.fetch(mode);

    self.a = self.y;

    self.flags.zero = self.a == 0;
    self.flags.negative = self.a & 0x80 != 0;
  }

  // endregion: Instructions

  pub fn reset(&mut self) {
    self.current_address_abs = 0xFFFC;
    let low = self.read(self.current_address_abs) as u16;
    let high = self.read(self.current_address_abs + 1) as u16;
    self.pc = (high << 8) | low;

    self.a = 0;
    self.x = 0;
    self.y = 0;
    self.sp = 0xFD;
    self.flags = Default::default();

    self.current_address_abs = 0x0000;
    self.current_address_rel = 0x0000;
    self.fetched_data = 0x00;

    self.cycles = 8;
  }

  pub fn irq(&mut self) {
    if !self.flags.interrupt_disable {
      self.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
      self.sp -= 1;
      self.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
      self.sp -= 1;

      self.flags.break_command = false;
      self.flags.interrupt_disable = true;

      self.write(0x0100 + self.sp as u16, self.flags.to_u8());
      self.sp -= 1;

      self.current_address_abs = 0xFFFE;
      let low = self.read(self.current_address_abs) as u16;
      let high = self.read(self.current_address_abs + 1) as u16;
      self.pc = (high << 8) | low;

      self.cycles = 7;
    }
  }

  pub fn nmi(&mut self) {
    self.write(0x0100 + self.sp as u16, (self.pc >> 8) as u8);
    self.sp -= 1;
    self.write(0x0100 + self.sp as u16, (self.pc & 0x00FF) as u8);
    self.sp -= 1;

    self.flags.break_command = false;
    self.flags.interrupt_disable = true;

    self.write(0x0100 + self.sp as u16, self.flags.to_u8());
    self.sp -= 1;

    self.current_address_abs = 0xFFFA;
    let low = self.read(self.current_address_abs) as u16;
    let high = self.read(self.current_address_abs + 1) as u16;
    self.pc = (high << 8) | low;

    self.cycles = 8;
  }
}