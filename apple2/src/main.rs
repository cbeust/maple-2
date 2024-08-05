mod memory;
mod debug;
mod constants;
mod apple2_cpu;
mod messages;
mod rolling_times;
mod test;
mod test_disk_controller;
mod cycle_actions;
mod misc;
mod alog;
mod test_memory;
pub mod roms;
mod memory_constants;
mod macros;
mod mini_fb;
mod config_file;
mod smartport;

mod disk {
    pub mod disk_controller;
    pub mod disk;
    pub mod drive;
    pub mod dsk;
    pub mod woz;
    mod woz_test;
    pub mod bit_stream;
    pub mod lss;
    pub mod dsk_to_woz;
    pub mod disk_info;
}


mod ui {
    pub mod soft_switches;
    pub mod text_screen;
    pub mod hires_screen;
    mod test_graphics;

    pub mod iced {
        pub mod ui_iced;
        pub mod message;
        mod disks_tab;
        mod nibbles_tab;
        mod memory_view;
        mod style;
        mod tab;
        mod keyboard;
        mod debugger_window;
        mod main_window;
        pub mod shared;
        mod disk_tab;
        mod drives_view;
    }
}


use cpu::messages::ToLogging;
use crossbeam::channel::{Receiver, Sender, unbounded};
use std::{fs, io, thread};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use cpu::cpu::Cpu;
use cpu::memory::Memory;
use messages::ToUi;
use clap::Parser;
use log4rs::append::Append;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;
use log::LevelFilter;
use notify::{RecursiveMode};
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use serde::{Serialize};
use cpu::config::{Config};
use cpu::logging_thread::Logging;
use crate::apple2_cpu::{AppleCpu, EmulatorConfigMsg};
use crate::config_file::ConfigFile;
use crate::constants::*;
use crate::disk::disk::Disk;
use crate::disk::disk_info::DiskInfo;
use crate::memory::{Apple2Memory};
use crate::messages::*;
use crate::messages::ToCpu::FileModified;
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::shared::Shared;
// use crate::ui::ui_egui::{MyEguiApp};
use crate::ui::iced::ui_iced::main_iced;

// pub fn is_text_hole(address: u16) -> bool {
//         (address >= 0x478 && address <= 0x47f) ||
//         (address >= 0x4f8 && address <= 0x4ff) ||
//         (address >= 0x578 && address <= 0x57f) ||
//         (address >= 0x5f8 && address <= 0x5ff) ||
//         (address >= 0x678 && address <= 0x67f) ||
//         (address >= 0x6f8 && address <= 0x6ff) ||
//         (address >= 0x778 && address <= 0x77f) ||
//         (address >= 0x7f8 && address <= 0x7ff)
// }

fn main() {
    start(true /* egui */);
    // start(false /* iced */);
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    rom: Option<String>,

    #[arg(short, long)]
    dir: Option<String>,
}

pub fn configure_log(config: &Config, remove: bool) -> log4rs::Config {
    let file_name = if config.csv {
        &config.trace_file_csv
    } else {
        &config.trace_file_asm
    };
    if config.trace_to_file {
        println!("Log to file enabled: {}", file_name);
    }
    if remove && Path::new(&file_name).exists() {
        fs::remove_file(file_name).unwrap();
    }

    let (appender_name, appender) : (&str, Box<dyn Append>) =
        if config.trace_to_file {
            println!("Tracing to file {}", file_name);
            let appender = FileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{m}\n")))
                .build(&file_name).unwrap();
            ("logfile", Box::new(appender))
        } else {
            if config.debug_asm {
            println!("Tracing to stdout");
            }
            // https://docs.rs/log4rs/1.0.0/log4rs/encode/pattern/index.html#formatters
            let appender = ConsoleAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{m}\n")))
                // .encoder(Box::new(PatternEncoder::new("{d(%H:%M:%SS)}: {m}\n")))
                .build();
            ("stdout", Box::new(appender))
        };

    log4rs::Config::builder()
        .appender(Appender::builder().build(appender_name, appender))
        .build(Root::builder()
            .appender(appender_name)
            .build(LevelFilter::Info))
        .unwrap()
}

// fn configure_log() -> log4rs::Config {
//     println!("Log to stdout is enabled");
//
//     let stdout = ConsoleAppender::builder()
//         // .encoder(Box::new(PatternEncoder::new("{disk(%f)} - {m}\n")))
//         .encoder(Box::new(PatternEncoder::new("{m}\n")))
//         .build();
//
//     log4rs::Config::builder()
//         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//         .build(Root::builder()
//             .appender("stdout")
//             .build(LevelFilter::Info))
//         .unwrap()
// }

fn t() {
    match disk::dsk_to_woz::dsk_to_woz("d:\\Apple disks\\Apple DOS 3.3.dsk") {
        Ok(f) => {
            println!("Wrote {f}");
        }
        Err(err) => {
            println!("ERROR: {}", err);
        }
    }
    println!("Correct size");
    disk::dsk_to_woz::woz_to_dsk("d:\\Apple disks\\Apple DOS 3.3.woz");

    println!("My size");
    if let Ok(f) = disk::dsk_to_woz::woz_to_dsk("c:\\Users\\Ced\\rust\\sixty.rs\\bad.woz") {
        println!("Wrote {f}");
    }
}

fn start(egui: bool) {
    let config_file = ConfigFile::new();
    let mut config = Config {
        emulator_speed_hz: config_file.emulator_speed_hz(),
        ..Default::default()
    };
    ui_log(&format!("Set speed of emulator to {} Mhz",
        (config.emulator_speed_hz as f32 / 1_000_000.0)));

    // t();
    // exit(0);
    for wf in WATCHED_FILES.iter() {
        config.watched_files.push(wf.clone());
    }

    let logger_config = configure_log(&config, true /* remove */);
    let handle = log4rs::init_config(logger_config).unwrap();

    START.set(Instant::now()).unwrap();

    log::info!("[Info] Logging");
    log::debug!("[Debug] Logging");

    //
    // Main emulator
    //
    let (sender, receiver): (Sender<ToUi>, Receiver<ToUi>) = unbounded();
    let _ = SENDER_TO_UI.set(sender.clone());
    let (sender2, receiver2): (Sender<ToCpu>, Receiver<ToCpu>) = unbounded();
    let benchmark = false;
    let mut disks = {
        let to_di = |drive: Option<String>| {
            let di = drive.map(|s| Disk::new(&s, false, Some(sender.clone())));
            if let Some(di2) = di {
                match di2 {
                    Ok(di3) => {
                        Some(di3.disk_info().clone())
                    }
                    Err(..) => {
                        None
                    }
                }
            } else {
                None
            }
        };

        [
            to_di(config_file.drive_1()),
            to_di(config_file.drive_2()),
        ]
    };
    let emulator_config = EmulatorConfigMsg::new(config.copy(), config_file.clone());
    let (logging_sender, logging_receiver): (Sender<ToLogging>, Receiver<ToLogging>) = unbounded();

    Shared::set_drive(0, disks[0].clone());
    Shared::set_drive(1, disks[1].clone());

    if benchmark {
        let mut apple2 = create_apple2(
            Some(sender), Some(logging_sender),
            Some(receiver2), disks, Box::new(emulator_config.clone()), None);
        apple2.cpu.run();
    } else {
        let sender4 = sender2.clone();
        let config_file_minifb = config_file.clone();

        //
        // Spawn the logging thread
        //
        let (sender_to_cpu_ui, receiver_to_cpu_ui) = unbounded();
        let config3 = config.clone();
        let config4 = config.clone();
        let _ = thread::Builder::new().name("Maple // - Logger".to_string()).spawn(move || {
            Logging::new(config3, logging_receiver, Some(sender_to_cpu_ui)).run();
        });

        //
        // Spawn the emulator
        //
        let _ = thread::Builder::new().name("Maple // - Emulator".to_string()).spawn(move || {
            let mut state = CpuStateMsg::Running;
            while state != CpuStateMsg::Exit { // running != CpuStateMsg::Paused {
                let ecm = EmulatorConfigMsg {
                    config: config4.clone(),
                    config_file: ConfigFile::new(),
                };
                let mut apple2 = create_apple2(Some(sender.clone()),
                    Some(logging_sender.clone()),
                    Some(receiver2.clone()), disks.clone(), Box::new(ecm), Some(handle.clone()));
                // if audit {
                //     apple2.cpu.cpu.memory.load_file("/Users/Ced/rust/a2audit/audit/audit.o", 0x6000, 0, 0, true);
                //     apple2.cpu.cpu.pc = 0x6000;
                // }
                for wf in WATCHED_FILES.iter() {
                    sender4.send(FileModified(wf.clone())).unwrap();
                }
                while state != CpuStateMsg::Rebooting && state != CpuStateMsg::Exit {
                    state = apple2.cpu.run();
                }
                println!("Exiting loop, status: {:#?}", state);
                disks = apple2.disks();
                if state != CpuStateMsg::Exit {
                    state = CpuStateMsg::Running;
                }
            }
            let _ = logging_sender.send(ToLogging::Exit);
            println!("Emulator exiting");
        });

        //
        // The minifb thread
        //
        let (sender_minifb, receiver_minifb): (Sender<ToMiniFb>, Receiver<ToMiniFb>) = unbounded();
        #[cfg(feature = "minifb")]
        let _ = thread::Builder::new().name("Maple // - minifb".to_string()).spawn(move || {
            mini_fb::main_minifb(receiver_minifb, &config_file);
        });

        //
        // The file watcher thread
        //
        if true {
            let config2 = config.copy();
            let sender3 = sender2.clone();
            let _ = thread::Builder::new().name("Maple // - File watcher".to_string()).spawn(move || {
                // Add a path to be watched. All files and directories at that path and
                // below will be monitored for changes.
                let (tx, rx) = std::sync::mpsc::channel();
                let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();
                for wf in config2.watched_files {
                    let path = &wf.path;
                    if !Path::new(&path).exists() {
                        println!("Path {} doesn't exist, ignoring", path);
                    } else {
                        debouncer.watcher().watch(path.as_ref(), RecursiveMode::NonRecursive).unwrap();
                        sender3.send(FileModified(wf.clone())).unwrap();
                        println!("Sending FileModified {}", wf.clone().path);
                    }
                }

                for res in rx {
                    match res {
                        Ok(events) => {
                            for event in events {
                                match event.kind {
                                    DebouncedEventKind::Any => {
                                        let wf = config.watched_files.iter()
                                            .find(|wf| wf.path == event.path.to_str().unwrap()).unwrap();
                                        sender3.send(FileModified(wf.clone())).unwrap();
                                        println!("Debounced event");
                                    }
                                    DebouncedEventKind::AnyContinuous => { println!("Any event") }
                                    _ => { println!("Unknownn event"); }
                                }
                                log::info!("Change: {event:?}")
                            }
                        },
                        Err(error) => log::error!("Error: {error:?}"),
                    }
                }
            });
        }

        //
        // Main UI
        //
        // if true {
        println!("Running iced");
        _ = main_iced(Some(sender2), receiver, Some(sender_minifb),
            config_file_minifb.clone());
    }
}

struct Apple2 {
    cpu: AppleCpu,
}

impl Apple2 {
    fn disks(&self) -> [Option<DiskInfo>; 2] {
        self.cpu.cpu.memory.disk_controller.disks()
    }
}

pub(crate) fn create_apple2(
    sender: Option<Sender<ToUi>>,
    logging_sender: Option<Sender<ToLogging>>,
    receiver: Option<Receiver<ToCpu>>,
    disk_infos: [Option<DiskInfo>; 2],
    config: Box<EmulatorConfigMsg>,
    handle: Option<Handle>)
-> Apple2
{
    let di0 = config.config_file.hard_drive_1().map(|s| DiskInfo::n(&s));
    let di1 = config.config_file.hard_drive_2().map(|s| DiskInfo::n(&s));

    let mut m = Apple2Memory::new(disk_infos, [di0, di1], sender.clone());

    m.load_roms(config.config_file.rom_type());

    let mut cpu = AppleCpu::new(Cpu::new(m, logging_sender, config.config.clone()),
        config.clone(), sender.clone(), receiver, handle);
    cpu.cpu.pc = cpu.cpu.memory.word(0xfffc);
    send_message!(sender, ToUi::Config(config.clone()));
    Apple2 { cpu }
}

/// Will need to log this into the UI somewhere
pub fn ui_log(s: &str) {
    println!("{}", s);
}

pub fn soft_switch(memory: &[u8], address: u16) -> bool {
    (memory[address as usize] & 0x80) != 0
}
