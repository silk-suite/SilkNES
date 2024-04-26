mod bus;
mod cartridge;
mod cpu;
mod ppu;

use bus::Bus;
use cartridge::Cartridge;
use cpu::NES6502;
use ppu::PPU;

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Create bus
    let bus = Rc::new(RefCell::new(Bus::new()));

    // Create CPU
    let cpu = Rc::new(RefCell::new(NES6502::new()));

    // Connect CPU to bus
    {
        let mut bus_ref = bus.borrow_mut();
        let cpu_ref = Rc::clone(&cpu);
        bus_ref.connect_cpu(Rc::clone(&cpu_ref));
    }

    // Connect bus to CPU
    {
        let mut cpu_ref = cpu.borrow_mut();
        let bus_ref = Rc::clone(&bus);
        cpu_ref.connect_to_bus(Rc::clone(&bus_ref));
    }

    // Create cartridge
    let cartridge = Rc::new(RefCell::new(Cartridge::from_rom("./roms/test/nestest.nes")));
    {
        let mut bus_ref = bus.borrow_mut();
        let cartridge_ref = Rc::clone(&cartridge);
        bus_ref.insert_cartridge(Rc::clone(&cartridge_ref));
    }

    //println!("{:?}", cartridge.borrow().prg_rom);
    cpu.borrow_mut().reset();
    // Skip to $C000 since no graphics yet
    cpu.borrow_mut().pc = 0xC000;

    // Call step() on the CPU
    for _ in 0..26000 {
        cpu.borrow_mut().step();
    }

    bus.borrow().dump_ram();
}