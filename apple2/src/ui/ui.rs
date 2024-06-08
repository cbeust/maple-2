use std::collections::{VecDeque};
use std::fs::{DirEntry, File};
use crossbeam::channel::{Receiver, Sender};
use std::time::{SystemTime};

use eframe::{Frame};
use eframe::egui::*;
use egui_virtual_list::VirtualList;
use cpu::config::Config;
use cpu::messages::ToCpuUi;
use crate::apple2_cpu::EmulatorConfigMsg;
use crate::ConfigFile;
use crate::messages::{CpuDumpMsg, SetMemoryMsg, ToCpu, ToMiniFb, TrackSectorMsg};
use crate::constants::*;
use crate::keyboard::key_to_char;
use crate::messages::ToCpu::*;
use crate::messages::ToUi;
use crate::disk::disk::Disk;
use crate::disk::disk_info::DiskInfo;
use crate::disk::drive::{DriveStatus};
use crate::ui::container::{Container};
use crate::memory_constants::*;
use crate::messages::ToMiniFb::Buffer;
use crate::ui::disks_window::DisplayedDisk;
use crate::ui::hires_screen::{AColor, HiresScreen};

#[derive(Clone, Copy, Default, PartialEq)]
pub enum DiskTab {
    #[default]
    Drives,
    Disks,
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum MainTab {
    #[default]
    Disks,
    Dev,
    Memory,
    Nibbles,
    Floppy,
    Debugger,
}

impl MainTab {
    pub(crate) fn to_index(self) -> usize {
        match self {
            MainTab::Disks => { 0 }
            MainTab::Memory => { 1 }
            MainTab::Nibbles => { 2 }
            MainTab::Floppy => { 3 }
            MainTab::Debugger => { 4 }
            MainTab::Dev => { 5 }
        }
    }

    fn calculate_tab(n: usize) -> MainTab {
        match n {
            1 => { MainTab::Memory }
            2 => { MainTab::Nibbles }
            3 => { MainTab::Floppy }
            4 => { MainTab::Debugger }
            5 => { MainTab::Dev }
            _ => { MainTab::Disks }
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum MemoryTab {
    #[default]
    Main, Text, TextAux, Graphic, GraphicAux,
}

#[derive(Clone, Copy)]
pub enum DrawCommand {
    // x0, y0, x1, y1, color
    Rectangle(f32, f32, f32, f32, AColor),
}

pub struct MyEguiApp {
    pub(crate) sender: Sender<ToCpu>,
    receiver: Receiver<ToUi>,
    receiver_cpu_ui: Option<Receiver<ToCpuUi>>,
    logging_status: String,
    sender_minifb: Option<Sender<ToMiniFb>>,
    pub cpu: CpuDumpMsg,
    pub(crate) config_file: ConfigFile,
    _last_update: SystemTime,
    pub(crate) memory_address: String,
    /// Last time we stepped the debugger (ms)
    last_step: SystemTime,
    /// How many cycles we stepped
    cycles_run: u64,
    /// Speed of the emulator in Mhz
    speed: f32,

    /// Used in double hires to keep track of the color of the last cell
    pub(crate) last_cell_is_color: bool,
    /// Used in double hires to keep track of the last bit written
    pub(crate) last_bit: usize,


    /// Keys received and not sent yet
    keys: VecDeque<u8>,
    key_sent: bool,

    /// DiskInfo of the disks inserted in the drives
    pub(crate) disk_infos: [Option<DiskInfo>; 2],
    pub(crate) drive_statuses: [DriveStatus; 2],
    pub(crate) track_sectors: [TrackSectorMsg; 2],

    /// Currently selected disk (0 or 1)
    pub(crate) disk_index: usize,

    hires_screen: HiresScreen,

    pub(crate) config: EmulatorConfigMsg,

    /// UI state
    selected_disk_tab: DiskTab,
    selected_main_tab: MainTab,
    pub(crate) selected_memory_tab: MemoryTab,

    pub(crate) min_width: f32,
    min_height: f32,

    /// Nibbles window
    pub(crate) nibbles_disk: Option<Disk>,
    pub(crate) nibbles_path: String,
    pub(crate) nibbles_selected_phase: usize,

    /// Debugger window
    pub(crate) debugger_emulator_paused: bool,
    pub(crate) debugger_a: String,
    pub(crate) debugger_x: String,
    pub(crate) debugger_y: String,
    pub(crate) debugger_pc: String,
    pub(crate) debugger_scroll_offset: Vec2,
    pub(crate) debugger_virtual_list: VirtualList,
    // The memory address currently highlighted. If none, defaults to PC
    pub(crate) debugger_current_line: Option<u16>,

    /// Disks window
    /// The content of the directory
    pub(crate) disks_all_disks: Vec<DisplayedDisk>,
    /// The disks we are actually displaying (might be filtered)
    pub(crate) disks_displayed_disks: Vec<DisplayedDisk>,
    pub(crate) disks_filter: String,

    /// If true, activate trace
    pub(crate) trace: bool,
    pub(crate) disassemble_from: String,
    pub(crate) disassemble_to: String,
    pub(crate) disassemble_to_file: String,

    /// If true, activate DHGR (debug)
    debug_dhgr: bool,

    /// If true, activate scan lines
    pub(crate) scan_lines: bool,

    /// Double hires graphics
    /// RGB mode (0-3)
    pub(crate) dhg_rgb_mode: u8,

}

impl MyEguiApp {
    fn send_next_key(&mut self) {
        if ! self.key_sent {
            if let Some(key) = self.keys.pop_front() {
                self.set_memory(0xc000, key);
                self.key_sent = true;
            }
        } else {
            self.key_sent = false;
        }
    }

    pub(crate) fn reg(ui: &mut Ui, label: &str, variable: &mut String) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(FontId::monospace(16.0)));
            ui.add(TextEdit::singleline(variable)
                .text_color(Color32::YELLOW)
                .font(FontId::monospace(16.0))
                .desired_width(60.0));
        });
    }

    pub(crate) fn label(ui: &mut Ui, label: &str, variable: String) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).font(FontId::monospace(12.0)));
            ui.label(RichText::new(variable));
        });
    }

    fn display(&mut self, ctx: &Context, _eframe: &mut Frame) {
        let logging_status = if self.logging_status.is_empty() { "".to_string() }
            else { format!(" - {}", self.logging_status) };
        ctx.send_viewport_cmd(ViewportCommand::Title(
            format!("Maple // - An Apple ][ emulator by Cédric Beust - Speed {:.2} Mhz {}",
                self.speed,
                logging_status
            )));
        let mut container = Container::default();

        // ctx.style_mut(|ui| ui.override_text_style = Some(TextStyle::Heading));

        container.add_window(ctx, "Drives", |ui: &mut Ui| {
            self.create_drives_window(ui);
        });
        container.add_window(ctx, "Actions", |ui: &mut Ui| {
            ui.set_min_width(self.min_width);
            ui.horizontal(|ui| {
                if ui.button("Reboot").clicked() {
                    ui_log("Rebooting");
                    self.on_reboot();
                    self.sender.send(Reboot).unwrap();
                }
                if ui.button("Swap").clicked() {
                    ui_log("Swapping the drives");
                    self.sender.send(SwapDisks).unwrap();
                }
                // if ui.button("Random disk").clicked() {
                //     let len = self.disks_all_disks.len();
                //     let d = thread_rng().gen_range(0..len);
                //     let disk_info = &self.disks_all_disks[d];
                //     self.sender.send(LoadDisk(0, disk_info.clone())).unwrap();
                // }
                if ui.button("Save HGR").clicked() {
                    self.sender.send(SaveGraphics).unwrap();
                }
                if ui.button("Debug").clicked() {
                    self.sender.send(Debug).unwrap();
                }
                ui.checkbox(&mut self.debug_dhgr, "DHGR");
            });
        });

        container.add_window(ctx, "Main", |ui: &mut Ui| {
            // ui.set_min_width(self.min_width);
            ui.set_max_width(self.min_width);
            ui.vertical(|ui| {
                create_tab_header(ui, &mut self.selected_main_tab, &mut self.config_file, &mut[
                    (MainTab::Disks, "Disks", None),
                    (MainTab::Memory, "Memory", None),
                    (MainTab::Nibbles, "Nibbles", None),
                    (MainTab::Floppy, "Floppy", None),
                    (MainTab::Debugger, "Debugger", None),
                    (MainTab::Dev, "Dev", None),
                ]);
                ui.separator();
                ui.set_min_height(self.min_height);
                match self.selected_main_tab {
                    MainTab::Disks => { self.create_disks_window(ui); }
                    MainTab::Dev => { self.create_dev_window(ui); }
                    MainTab::Memory => { self.create_memory_window(ui); }
                    MainTab::Nibbles => { self.create_nibble_window(ui); }
                    MainTab::Debugger => { self.create_debugger_window(ui); }
                    MainTab::Floppy => { self.create_floppy_window(ui); }
                }
            });
        });

        fn create_tab_header(ui: &mut Ui, current: &mut MainTab,
                config_file: &mut ConfigFile,
                values: &mut [(MainTab, &str, Option<&mut bool>)]) {
            // ui.style_mut().spacing.item_spacing = Vec2::new(20.0, 10.0);
            // ui.set_style(*s);
            ui.horizontal(|ui| {
                // let c = values[0].2.as_mut().unwrap();
                // ui.checkbox(c, "");
                for i in 0..values.len() {
                    let layout = if i < values.len() - 1 {
                        Layout::left_to_right(Align::Center)
                    } else {
                        Layout::right_to_left(Align::Center)
                    };
                    ui.with_layout(layout, |ui| {
                        let (value, title, ref mut checked) = values[i];
                        // ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.selectable_value(current, value,
                                RichText::new(title.to_string())
                                        .color(Color32::LIGHT_GREEN).strong()).clicked() {
                                    config_file.set_tab(*current);
                            }; // .size(20.0));
                            if let Some(c) = checked.as_mut() {
                                ui.checkbox(c, "");
                            }
                        });
                        // ui.add_space(20.0);
                        ui.separator();
                    });
                }
            });
            ui.reset_style();
        }

        let frame = containers::Frame {
            inner_margin: Margin { left: 10., right: 10., top: 10., bottom: 10. },
            outer_margin: Margin { left: 10., right: 10., top: 10., bottom: 10. },
            rounding: Rounding { nw: 1.0, ne: 1.0, sw: 1.0, se: 1.0 },
            shadow: epaint::Shadow { extrusion: 1.0, color: Color32::RED },
            fill: Color32::BLACK,
            stroke: Stroke::new(10.0, Color32::BLACK),
        };

        CentralPanel::default().frame(frame).show(ctx, |ui| {
            //
            // Retrieve the DrawCommands based on the various display modes
            //
            let mut draw_commands: Vec<DrawCommand> = Vec::new();
            if ! self.cpu.memory.is_empty() {
                if self.is_dhgr_on() {
                    draw_commands = self.calculate_double_hires2(false /* page1 */);
                } else {
                    let page2 = soft_switch(&self.cpu.memory, PAGE_2_STATUS);
                    let is_text = soft_switch(&self.cpu.memory, TEXT_STATUS);
                    let is_hires = soft_switch(&self.cpu.memory, HIRES_STATUS);
                    let is_mixed = soft_switch(&self.cpu.memory, MIXED_STATUS);
                    let is_80 = soft_switch(&self.cpu.memory, EIGHTY_COLUMNS_STATUS);
                    let alt_charset = soft_switch(&self.cpu.memory, ALT_CHAR_STATUS);
                    if is_text {
                        draw_commands = self.calculate_text(is_80, false /* not mixed */,page2,
                            alt_charset);
                    } else if is_hires {
                        if is_mixed {
                            draw_commands = self.calculate_hires(HIRES_HEIGHT_MIXED, page2);
                            draw_commands.append(&mut self.calculate_text(is_80, true /* mixed */,
                                page2, alt_charset));
                        } else {
                            draw_commands = self.calculate_hires(HIRES_HEIGHT, page2);
                        }
                    }
                }
            };

            // If you want to have some fun with display stuff, uncomment this
            // draw_commands = draw_commands.iter().map(|dc| {
            //     match dc {
            //         DrawCommand::Rectangle(x0, y0, x1, y1, color) => {
            //             DrawCommand::Rectangle(
            //                 (HIRES_WIDTH * MAGNIFICATION) as f32 - *x0,
            //                 (HIRES_HEIGHT * MAGNIFICATION) as f32 - *y0,
            //                 (HIRES_WIDTH * MAGNIFICATION) as f32 - *x1,
            //                 (HIRES_HEIGHT * MAGNIFICATION) as f32 - *y1,
            //                 *color)
            //         }
            //     }
            // }).collect::<Vec<DrawCommand>>();

            // Only needed if we want minifb
            let mut draw_commands2: Vec<DrawCommand> = Vec::new();

            //
            // Actually render the DrawCommands we received from the back end
            //
            for r in draw_commands {
                #[cfg(feature = "minifb")]
                draw_commands2.push(r);
                match r {
                    DrawCommand::Rectangle(x0, y0, x1, y1, color) => {
                        let re = Rect::from_points(&[
                            Pos2::new(x0, y0), Pos2::new(x1, y1)
                        ]);
                        ui.painter().rect_filled(re, 0.0, color);
                    }
                }
            }

            #[cfg(feature = "minifb")]
            if let Some(sender_minifb) = &self.sender_minifb {
                sender_minifb.send(Buffer(draw_commands2)).unwrap();
            }

            // CRT effect (scan lines), add horizontal black line every other line
            // (unfortunately, makes the display a lot darker)
            if self.scan_lines {
                for i in (0..HIRES_HEIGHT * MAGNIFICATION).step_by(4) {
                    let r = Rect::from_points(&[
                        Pos2::new(0.0, i as f32),
                        Pos2::new((HIRES_WIDTH * MAGNIFICATION) as f32, i as f32)
                    ]);
                    ui.painter().rect(r, 0.0, Color32::BLACK, Stroke {
                        width: 1.0,
                        color: Color32::BLACK,
                    });
                }
            }

            ui.input(|i| {
                for event in &i.events {
                    match event {
                        Event::Text(s) => {
                            // for c in s.chars() {
                            //     self.keys.push_back((c.to_ascii_uppercase() as u8) + 0x80);
                            // }
                        }
                        Event::Key{key, pressed, repeat: _repeat, modifiers, ..} => {
                            if ! pressed {
                                let c = key_to_char(key, modifiers);
                                self.keys.push_back(c);
                            } else {
                                self.set_memory(0xc064, 0x80);
                                self.set_memory(0xc065, 0x80);
                            }
                        }
                        _ => {}
                    }
                }
            });
        });
    }

    fn set_memory(&mut self, address: u16, byte: u8) {
        self.sender.send(SetMemory(SetMemoryMsg {
            address, bytes: vec![byte],
        })).unwrap();
    }

    pub(crate) fn new(config_file: ConfigFile, _: &eframe::CreationContext<'_>,
        sender: Sender<ToCpu>, receiver: Receiver<ToUi>, receiver_cpu_ui: Option<Receiver<ToCpuUi>>,
                sender_minifb: Option<Sender<ToMiniFb>>)
            -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let disks = Self::read_disks_directories(config_file.disk_directories());
        let default_tab = MainTab::calculate_tab(config_file.tab);

        Self {
            _last_update: SystemTime::now(),
            cpu: CpuDumpMsg::default(),
            sender,
            receiver,
            receiver_cpu_ui,
            sender_minifb,
            config_file,
            keys: VecDeque::with_capacity(10),
            key_sent: false,
            memory_address: "0000".to_string(),
            last_step: SystemTime::now(),
            cycles_run: 0,
            speed: 0.0,
            hires_screen: HiresScreen::new(),
            disk_infos: [ None, None ],
            track_sectors: [TrackSectorMsg::default(), TrackSectorMsg::default()],
            drive_statuses: [ DriveStatus::Off, DriveStatus::Off ],
            disk_index: 0,
            last_cell_is_color: true,
            last_bit: 0,
            selected_disk_tab: DiskTab::Disks,
            selected_main_tab: default_tab,
            selected_memory_tab: MemoryTab::TextAux,
            min_width: UI_WIDTH,
            min_height: 600.0,
            config: EmulatorConfigMsg::default(),
            nibbles_disk: None,
            nibbles_path: "".to_string(),
            nibbles_selected_phase: 0,
            debugger_emulator_paused: false,
            debugger_a: "".to_string(),
            debugger_x: "".to_string(),
            debugger_y: "".to_string(),
            debugger_pc: "".to_string(),
            debugger_scroll_offset: Vec2::default(),
            debugger_virtual_list: VirtualList::new(),
            debugger_current_line: None,
            disks_displayed_disks: disks.clone(),
            disks_all_disks: disks,
            disks_filter: "".to_string(),

            trace: false,
            disassemble_from: "0".to_string(),
            disassemble_to: "0".to_string(),
            disassemble_to_file: Config::default().trace_file_asm,
            debug_dhgr: false,

            dhg_rgb_mode: 0,
            scan_lines: true,

            logging_status: "".to_string(),
        }
    }

    /// DHGR is set by writing in no particular order:
    /// C0..:
    /// 5E (AN3)
    /// 0d (80 VID)
    /// 50 (graphics)
    /// 52 (full screen)
    /// 54 (page1)
    /// 57 (hires)
    fn is_dhgr_on(&self) -> bool {
        let an3 = self.cpu.memory[AN3_STATUS as usize] & 0b0010_0000 != 0;
        let eighty = soft_switch(&self.cpu.memory, EIGHTY_COLUMNS_STATUS);
        let is_graphics = ! soft_switch(&self.cpu.memory, TEXT_STATUS);
        let full_screen = ! soft_switch(&self.cpu.memory, MIXED_STATUS);
        let is_hires = soft_switch(&self.cpu.memory, HIRES_STATUS);

        an3 && eighty && is_graphics && full_screen && is_graphics && is_hires
    }

    fn on_reboot(&mut self) {
        self.dhg_rgb_mode = 0;
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &Context, eframe: &mut Frame) {
        use ToUi::*;
        if let Some(receiver) = &self.receiver_cpu_ui {
            while let Ok(message) = receiver.try_recv() {
                match message {
                    ToCpuUi::LogStarted => {
                        self.logging_status = "Logging active".to_string();
                    }
                    ToCpuUi::LogEnded => {
                        self.logging_status = "".to_string();
                    }
                }
            }
        }

        while let Ok(message) = self.receiver.try_recv() {
            match message {
                Config(config) => {
                    self.config = config;
                }
                CpuDump(cpu) => {
                    self.debugger_pc = format!("{:04X}", cpu.pc);
                    self.debugger_a = format!("{:02X}", cpu.a);
                    self.debugger_x = format!("{:02X}", cpu.x);
                    self.debugger_y = format!("{:02X}", cpu.y);
                    self.cpu = cpu;
                }
                EmulatorSpeed(s) => {
                    self.speed = s;
                }
                TrackSector(drive_index, msg) => {
                    self.track_sectors[drive_index] = msg;
                }
                PhaseUpdate(drive_index, phase80) => {
                    self.nibbles_selected_phase = (phase80 * 2) as usize;
                    self.track_sectors[drive_index].track = (phase80 / 2) as f32;
                }
                DiskInfo(drive_number, disk_info) => {
                    self.disk_infos[drive_number] = disk_info.clone();
                    if let Some(di) = &self.disk_infos[drive_number] {
                        ui_log(&format!(
                            "New disk inserted in drive {drive_number}: {}", &di.name()));
                    }
                    self.config_file.set_drive(drive_number, disk_info.map(|di| di.path));
                }
                KeyboardStrobe => {
                    self.send_next_key();
                }
                DriveMotorStatus(drive_index, status) => {
                    self.drive_statuses[drive_index] = status;
                }
                DiskSelected(disk_index) => {
                    self.disk_index = disk_index;
                }
                RgbModeUpdate(rgb_mode) => {
                    self.dhg_rgb_mode = rgb_mode;
                }
            }
        }

        self.send_next_key();
        self.display(ctx, eframe);
        ctx.request_repaint();
    }
}

fn soft_switch(memory: &[u8], address: u16) -> bool {
    (memory[address as usize] & 0x80) != 0
}

/// Will need to log this into the UI somewhere
pub fn ui_log(s: &str) {
    println!("{}", s);
}
