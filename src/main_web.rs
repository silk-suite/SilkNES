pub mod apu;
pub mod apu_output;
pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod ppu;
pub mod mapper;
pub mod mappers;

use apu::APU;
use apu_output::APUOutput;
use bus::{Bus, BusLike};
use cartridge::Cartridge;
use cpu::NES6502;
use ppu::PPU;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc,
    Mutex
};

use eframe::egui;
use egui::Key;
use rodio::{source::Source, OutputStream, Sink};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref HAS_ROM: AtomicBool = AtomicBool::new(false);
    static ref ROM_CHANGED: AtomicBool = AtomicBool::new(false);
    static ref ROM_BYTES: Mutex<Vec<u8>> = Mutex::new(vec![]);
    static ref CONTROLLER_STATE: Mutex<u8> = Mutex::new(0);
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    // Create bus
    let bus = Rc::new(RefCell::new(Box::new(Bus::new()) as Box<dyn BusLike>));

    // Create CPU
    let cpu = Rc::new(RefCell::new(NES6502::new()));

    let ppu = Rc::new(RefCell::new(PPU::new()));

    let apu = Rc::new(RefCell::new(APU::new()));

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

    // Connect bus to APU
    {
        let mut bus_ref = bus.borrow_mut();
        let apu_ref = Rc::clone(&apu);
        bus_ref.connect_apu(Rc::clone(&apu_ref));
    }

    // Connect APU to bus
    {
        let mut apu_ref = apu.borrow_mut();
        let bus_ref = Rc::clone(&bus);
        apu_ref.connect_to_bus(Rc::clone(&bus_ref));
    }

    // Setup audio
    let (tx, rx) = mpsc::channel();
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let _sink = Sink::try_new(&stream_handle).unwrap();
    let source = APUOutput::new(rx).amplify(0.25);
    _sink.append(source);

    let silknes = SilkNES {
        bus,
        cpu,
        ppu,
        apu,
        cartridge: None,
        rom_loaded: false,
        tx,
        _sink,
        _stream,
    };
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "nesCanvas", // hardcode it
                web_options,
                Box::new(|cc| Box::new(silknes)),
            )
            .await
            .expect("failed to start eframe");
    });
}

struct SilkNES {
    bus: Rc<RefCell<Box<dyn BusLike>>>,
    cpu: Rc<RefCell<NES6502>>,
    ppu: Rc<RefCell<PPU>>,
    apu: Rc<RefCell<APU>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    rom_loaded: bool,

    tx: mpsc::Sender<Vec<f32>>,
    _sink: Sink,
    _stream: OutputStream,
}

impl eframe::App for SilkNES {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        ctx.request_repaint();

        if !HAS_ROM.load(Ordering::Relaxed) {
            if ROM_CHANGED.load(Ordering::Relaxed) {
                ROM_CHANGED.store(false, Ordering::Relaxed);
                HAS_ROM.store(true, Ordering::Relaxed);
                let cartridge = Rc::new(RefCell::new(Cartridge::from_bytes(ROM_BYTES.lock().unwrap().to_owned())));
                {
                    let mut bus_ref = self.bus.borrow_mut();
                    let cartridge_ref = Rc::clone(&cartridge);
                    bus_ref.insert_cartridge(Rc::clone(&cartridge_ref));
                }
                self.cartridge = Some(cartridge);
                self.cpu.borrow_mut().reset();
                self.ppu.borrow_mut().reset();
                self.rom_loaded = true;
            } else {
              return;
            }
        }
        if self.rom_loaded {
            // Run the emulation
            // It would be nice to just eventually step the bus itself,
            // but the borrow checker is screwing me here so this is fine for now
            for _ in 0..(341*262) {
                // Grab some variables from the bus to use while stepping
                let cycles = self.bus.borrow().get_global_cycles();
                let dma_running = self.bus.borrow().dma_running();
                let mut should_run_dma = false;

                self.ppu.borrow_mut().step();
                if cycles % 3 == 0 {
                    if self.bus.borrow().dma_queued() && !dma_running {
                        if cycles % 2 == 1 {
                            should_run_dma = true;
                        }
                    } else if dma_running {
                        if cycles % 2 == 0 {
                            let dma_data = {
                                let bus = self.bus.borrow();
                                let dma_page = bus.dma_page() as u16;
                                let dma_address = bus.dma_address() as u16;
                                let dma_data = bus.cpu_read((dma_page << 8) | dma_address);
                                dma_data
                            };
                            self.bus.borrow_mut().set_dma_data(dma_data);
                        } else {
                            let mut dma_address = self.bus.borrow().dma_address();
                            let dma_data = self.bus.borrow().dma_data();
                            let oam_index = (dma_address / 4) as usize;
                            let mut ppu = self.ppu.borrow_mut();
                            match dma_address % 4 {
                                0 => ppu.oam[oam_index].y = dma_data,
                                1 => ppu.oam[oam_index].id = dma_data,
                                2 => ppu.oam[oam_index].attributes.set_from_u8(dma_data),
                                3 => ppu.oam[oam_index].x = dma_data,
                                _ => (),
                            }
                            dma_address = dma_address.wrapping_add(1);
                            self.bus.borrow_mut().set_dma_address(dma_address);

                            if dma_address == 0 {
                                self.bus.borrow_mut().set_dma_running(false);
                                self.bus.borrow_mut().set_dma_queued(false);
                            }
                        }
                    } else {
                        self.cpu.borrow_mut().step();
                        self.apu.borrow_mut().step(self.cpu.borrow().total_cycles);
                        if self.apu.borrow().registers.status.dmc_interrupt || self.apu.borrow().registers.status.frame_interrupt || self.cartridge.as_ref().unwrap().borrow().mapper.irq_state() {
                            self.cpu.borrow_mut().irq();
                        }
                    }
                }
                let nmi = self.ppu.borrow().nmi;
                if nmi {
                    self.ppu.borrow_mut().nmi = false;
                    self.cpu.borrow_mut().nmi();
                }
                self.bus.borrow_mut().set_global_cycles(cycles + 1);
                if should_run_dma {
                    self.bus.borrow_mut().set_dma_running(true);
                }
                // self.apu.borrow_mut().update_output();
            }

            // // Update audio
            // let buffer = std::mem::take(&mut self.apu.borrow_mut().output_buffer);
            // let averaged = buffer
            //     .chunks(112)
            //     .fold(Vec::new(), |mut acc, x| {
            //         let sum: f32 = x.iter().sum();
            //         acc.push(sum / x.len() as f32);
            //         acc
            //     });
            // self.tx.send(averaged).unwrap();
        }

        // Render the display to a texture for egui
        let display = self.ppu.borrow().get_screen();
        let color_image = egui::ColorImage::from_rgb([256, 240], &display);
        let handle = ctx.load_texture("Display", color_image, egui::TextureOptions::NEAREST);

        // Draw main window
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(512.0, 480.0));
            let image = egui::Image::from_texture(sized_image);
            ui.add(image);
        });

        // Handle input
        let mut controller_state = 0x00;
        let outside_state = *CONTROLLER_STATE.lock().unwrap();

        for (key, value) in [
            (Key::ArrowRight, 0x01), // D-Pad Right
            (Key::ArrowLeft, 0x02), // D-Pad Left
            (Key::ArrowDown, 0x04), // D-Pad Down
            (Key::ArrowUp, 0x08), // D-Pad Up
            (Key::Enter, 0x10), // Start
            (Key::Space, 0x20), // Select
            (Key::Z, 0x40), // B
            (Key::X, 0x80), // A
        ] {
            if ctx.input(|i| i.key_down(key) || outside_state & value == value) {
                controller_state |= value;
            }
        }
        self.bus.borrow_mut().update_controller(0, controller_state);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn load_rom(bytes: Vec<u8>) {
  ROM_BYTES.lock().unwrap().clear();
  ROM_BYTES.lock().unwrap().extend_from_slice(&bytes);
  ROM_CHANGED.store(true, Ordering::Relaxed);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn set_controller_state(value: u8) {
  *CONTROLLER_STATE.lock().unwrap() = value;
}
