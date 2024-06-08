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
use std::sync::mpsc;

use std::collections::HashMap;

use eframe::egui;
use egui::Key;
use muda::{accelerator::{Accelerator, Code, Modifiers}, Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem, Submenu};
use rfd::FileDialog;
use rodio::{source::Source, OutputStream, Sink};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

fn main() -> Result<(), eframe::Error> {
    // Set window options, main important one here is min_inner_size so our window accounts for menubar insertion
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([512.0, 480.0]).with_min_inner_size([512.0, 480.0]),
        ..Default::default()
    };

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
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = APUOutput::new(rx).amplify(0.25);
    sink.append(source);

    let silknes = SilkNES {
        show_about_window: false,
        menubar: None,
        menubar_items: HashMap::new(),
        menubar_interaction: "".to_string(),
        bus,
        cpu,
        ppu,
        apu,
        cartridge: None,
        rom_loaded: false,
        tx,
    };
    eframe::run_native(
        "SilkNES",
        options,
        Box::new(|_cc| Box::<SilkNES>::new(silknes)),
    )
}

struct SilkNES {
    /// Immediate viewports are show immediately, so passing state to/from them is easy.
    /// The downside is that their painting is linked with the parent viewport:
    /// if either needs repainting, they are both repainted.
    show_about_window: bool,

    menubar: Option<Menu>,
    menubar_items: HashMap<MenuId, String>,
    menubar_interaction: String,

    bus: Rc<RefCell<Box<dyn BusLike>>>,
    cpu: Rc<RefCell<NES6502>>,
    ppu: Rc<RefCell<PPU>>,
    apu: Rc<RefCell<APU>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    rom_loaded: bool,

    tx: mpsc::Sender<Vec<f32>>,
}

impl eframe::App for SilkNES {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        ctx.request_repaint();

        // Check for interactions on the menubar
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let item_string = self.menubar_items.get(event.id()).unwrap();
            match item_string.as_str() {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["nes", "fds"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        // TODO: Reset properly
                        let cartridge = Rc::new(RefCell::new(Cartridge::from_rom(path.to_str().unwrap())));
                        {
                            let mut bus_ref = self.bus.borrow_mut();
                            let cartridge_ref = Rc::clone(&cartridge);
                            bus_ref.insert_cartridge(Rc::clone(&cartridge_ref));
                        }
                        self.cartridge = Some(cartridge);
                        self.rom_loaded = true;

                        self.cpu.borrow_mut().reset();
                        self.ppu.borrow_mut().reset();
                    }
                },
                "Quit" => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                },
                "About" => {
                    self.show_about_window = true;
                }
                _ => {}
            }
        } else if self.menubar_interaction != "" {
            // I don't love this but it's conceptually easier than messing around
            // with the Windows API I'd have to interact with for accelerators
            match self.menubar_interaction.to_owned().as_str() {
                "Load ROM" => {
                    let file = FileDialog::new()
                        .add_filter("ROMs", &["nes", "fds"])
                        .set_directory("./roms")
                        .pick_file();
                    if let Some(path) = file {
                        // TODO: Reset properly
                        let cartridge = Rc::new(RefCell::new(Cartridge::from_rom(path.to_str().unwrap())));
                        {
                            let mut bus_ref = self.bus.borrow_mut();
                            let cartridge_ref = Rc::clone(&cartridge);
                            bus_ref.insert_cartridge(Rc::clone(&cartridge_ref));
                        }
                        self.cartridge = Some(cartridge);
                        self.rom_loaded = true;

                        self.cpu.borrow_mut().reset();
                        self.ppu.borrow_mut().reset();
                    }
                },
                _ => {}
            }
            self.menubar_interaction = "".to_string();
        }

        if self.rom_loaded {
            // Run the emulation
            // It would be nice to just eventually step the bus itself,
            // but the borrow checker is screwing me here so this is fine for now
            let mut audio_buffer = Vec::new();
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
                            let dma_page = self.bus.borrow().dma_page() as u16;
                            let dma_address = self.bus.borrow().dma_address() as u16;
                            let dma_data = self.bus.borrow().cpu_read((dma_page << 8) | dma_address);
                            self.bus.borrow_mut().set_dma_data(dma_data);
                        } else {
                            let mut dma_address = self.bus.borrow().dma_address();
                            let dma_data = self.bus.borrow().dma_data();
                            let oam_index = (dma_address / 4) as usize;
                            match dma_address % 4 {
                                0 => self.ppu.borrow_mut().oam[oam_index].y = dma_data,
                                1 => self.ppu.borrow_mut().oam[oam_index].id = dma_data,
                                2 => self.ppu.borrow_mut().oam[oam_index].attributes.set_from_u8(dma_data),
                                3 => self.ppu.borrow_mut().oam[oam_index].x = dma_data,
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
                audio_buffer.push(self.apu.borrow_mut().get_output());
            }

            // Update audio
            let averaged = audio_buffer
                .chunks(112)
                .map(|x| x.iter().sum::<f32>() / x.len() as f32)
                .collect::<Vec<f32>>();
            self.tx.send(averaged).unwrap();
            audio_buffer.clear();
        }

        // Render the display to a texture for egui
        let display = self.ppu.borrow().get_screen();
        let pixels = display
            .into_iter()
            .flatten()
            .collect::<Vec<u8>>();
        let color_image = egui::ColorImage::from_rgb([256, 240], &pixels);
        let handle = ctx.load_texture("Display", color_image, egui::TextureOptions::NEAREST);

        // Draw main window
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            if self.menubar.is_none() {
                let handle = _frame.window_handle().unwrap().as_raw();
                let hwnd = match handle {
                    RawWindowHandle::Win32(handle) => handle.hwnd.get(),
                    _ => panic!("Cannot handle other platform window handles yet!"),
                };
                let (menubar, menubar_items) = create_menubar();
                menubar.init_for_hwnd(hwnd).unwrap();
                self.menubar = Some(menubar);
                self.menubar_items = menubar_items;
            }

            let sized_image = egui::load::SizedTexture::new(handle.id(), egui::vec2(512.0, 480.0));
            let image = egui::Image::from_texture(sized_image);
            ui.add(image);
        });

        // Draw about window, if activve
        if self.show_about_window {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("about_window"),
                egui::ViewportBuilder::default()
                    .with_title("About")
                    .with_inner_size([256.0, 128.0]),
                |ctx, class| {
                    assert!(
                        class == egui::ViewportClass::Immediate,
                        "This egui backend doesn't support multiple viewports"
                    );

                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label("Created by Daniel Adams");
                        })
                    });

                    if ctx.input(|i| i.viewport().close_requested()) {
                        // Tell parent viewport that we should not show next frame:
                        self.show_about_window = false;
                    }
                },
            );
        }

        // Handle input
        let mut controller_state = 0x00;

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
            if ctx.input(|i| i.key_down(key)) {
                controller_state |= value;
            }

            self.bus.borrow_mut().update_controller(0, controller_state);

            if ctx.input(|i| i.modifiers.ctrl) && ctx.input(|i| i.key_pressed(Key::O)) {
                self.menubar_interaction = "Load ROM".to_string();
            }
        }

        if ctx.input(|i| i.modifiers.ctrl) && ctx.input(|i| i.key_pressed(Key::O)) {
            self.menubar_interaction = "Load ROM".to_string();
        }
    }
}

fn create_menubar() -> (Menu, HashMap<MenuId, String>) {
    let menu = Menu::new();

    // File Tab
    let load_rom = MenuItem::new(
        "Load ROM",
        true,
        Some(Accelerator::new(Some(Modifiers::CONTROL), Code::KeyO)),
    );
    let quit = MenuItem::new(
        "Quit",
        true,
        None,
    );
    let file_tab = Submenu::with_items(
        "File",
        true,
        &[
            &load_rom,
            &PredefinedMenuItem::separator(),
            &quit,
        ],
    ).unwrap();
    menu.append(&file_tab).unwrap();

    // Help Tab
    let about = MenuItem::new(
        "About",
        true,
        None,
    );
    let help_tab = Submenu::with_items(
        "Help",
        true,
        &[
            &about,
        ],
    ).unwrap();
    menu.append(&help_tab).unwrap();

    let mut menu_ids = HashMap::new();
    menu_ids.insert(load_rom.id().clone(), "Load ROM".to_string());
    menu_ids.insert(quit.id().clone(), "Quit".to_string());
    menu_ids.insert(about.id().clone(), "About".to_string());

    (menu, menu_ids)
}