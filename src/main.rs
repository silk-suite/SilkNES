pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod ppu;
pub mod mapper;
pub mod mapper0;

use bus::{Bus, BusLike};
use cartridge::Cartridge;
use cpu::NES6502;
use ppu::PPU;

use std::cell::RefCell;
use std::rc::Rc;

use pixels::{Pixels, SurfaceTexture};
//use rodio::{source::Source, OutputStream, Sink};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(512, 480))
        .with_title("NESilk")
        .build(&event_loop)
        .unwrap();
    let mut input = WinitInputHelper::new();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        let initial_width = 256;
        let initial_height = 240;
        Pixels::new(initial_width, initial_height, surface_texture).unwrap()
    };

    // Create bus
    let bus = Rc::new(RefCell::new(Box::new(Bus::new()) as Box<dyn BusLike>));

    // Create CPU
    let cpu = Rc::new(RefCell::new(NES6502::new()));

    let ppu = Rc::new(RefCell::new(PPU::new()));

    // Connect bus to CPU
    {
        let mut bus_ref = bus.borrow_mut();
        let cpu_ref = Rc::clone(&cpu);
        bus_ref.connect_cpu(Rc::clone(&cpu_ref));
    }

    // Connect CPU to bus
    {
        let mut cpu_ref = cpu.borrow_mut();
        let bus_ref = Rc::clone(&bus);
        cpu_ref.connect_to_bus(Rc::clone(&bus_ref));
    }

    // Connect bus to PPU
    {
        let mut bus_ref = bus.borrow_mut();
        let ppu_ref = Rc::clone(&ppu);
        bus_ref.connect_ppu(Rc::clone(&ppu_ref));
    }

    // Connect PPU to bus
    {
        let mut ppu_ref = ppu.borrow_mut();
        let bus_ref = Rc::clone(&bus);
        ppu_ref.connect_to_bus(Rc::clone(&bus_ref));
    }

    // Create cartridge
    let cartridge = Rc::new(RefCell::new(Cartridge::from_rom("./roms/test/nestest.nes")));
    {
        let mut bus_ref = bus.borrow_mut();
        let cartridge_ref = Rc::clone(&cartridge);
        bus_ref.insert_cartridge(Rc::clone(&cartridge_ref));
    }

    cpu.borrow_mut().reset();

    event_loop.set_control_flow(ControlFlow::Poll);
    
    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("The close button was pressed; stopping");
                elwt.exit();
            },
            Event::AboutToWait => {
                // Run the emulation
                // It would be nice to just eventually step the bus itself,
                // but the borrow checker is screwing me here so this is fine for now
                for _ in 0..(341*262) { // Up the clock speed
                    ppu.borrow_mut().step();
                    let cycles = bus.borrow().get_global_cycles();
                    if cycles % 3 == 0 {
                        cpu.borrow_mut().step();
                    }
                    let nmi = ppu.borrow().nmi;
                    if nmi {
                        ppu.borrow_mut().nmi = false;
                        cpu.borrow_mut().nmi();
                    }
                    bus.borrow_mut().set_global_cycles(cycles + 1);
                }

                // Draw to screen
                let display = ppu.borrow().get_screen();
                let frame = pixels.frame_mut();

                for (pixel, &value) in frame.chunks_mut(4).zip(display.iter()) {
                    pixel.copy_from_slice(&value);
                }

                if let Err(err) = pixels.render() {
                    println!("pixels.render() failed: {}", err);
                    elwt.exit();
                }
            },
            _ => ()
        }

        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
            }

            let mut controller_state = 0x00;

            for (key, value) in [
                (KeyCode::ArrowRight, 0x01), // D-Pad Right
                (KeyCode::ArrowLeft, 0x02), // D-Pad Left
                (KeyCode::ArrowDown, 0x04), // D-Pad Down
                (KeyCode::ArrowUp, 0x08), // D-Pad Up
                (KeyCode::Enter, 0x10), // Start
                (KeyCode::Space, 0x20), // Select
                (KeyCode::KeyZ, 0x40), // B
                (KeyCode::KeyX, 0x80), // A
            ] {
                if input.key_held(key) {
                    controller_state |= value;
                }
            }

            bus.borrow_mut().update_controller(0, controller_state);
        }
    });
}