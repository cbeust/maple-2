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
mod speaker;
mod joystick;

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
        mod debug_tab;
    }
}


use cpu::messages::ToLogging;
use crossbeam::channel::{Receiver, Sender, unbounded};
use std::{fs, io, thread};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{stdout};
use std::path::{Path};
use std::process::exit;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, Instant};
use cpu::cpu::Cpu;
use cpu::memory::Memory;
use messages::ToUi;
use clap::Parser;
use gilrs::{Axis, Event, Gilrs};
use notify::{RecursiveMode};
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tracing::{debug, error, event, info, Level, span, warn};
use tracing_subscriber::{EnvFilter, fmt, Registry, registry};
use tracing_subscriber::fmt::{format, Layer, MakeWriter, Subscriber};
use tracing_subscriber::fmt::format::{Compact, DefaultFields, Format};
use tracing_subscriber::layer::SubscriberExt;
use cpu::config::{Config};
use cpu::logging_thread::Logging;
use crate::apple2_cpu::{AppleCpu, EmulatorConfigMsg};
use crate::config_file::ConfigFile;
use crate::constants::*;
use crate::disk::disk::Disk;
use crate::disk::disk_info::DiskInfo;
use crate::joystick::Joystick;
use crate::memory::{Apple2Memory};
use crate::messages::*;
use crate::messages::ToCpu::FileModified;
use crate::speaker::{play_file_rodio, Speaker, Speaker2};
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::ui_iced::{main_iced};

fn configure_tracing(to_file: bool) {
    // A layer that logs events to a file
    let l = fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_level(false)       // include levels in the output
        .with_target(false)     // don't include event targets
        .with_thread_ids(false)
        .compact();
    if to_file {
        let file = File::create("c:\\t\\trace.txt").unwrap();
        tracing::subscriber::set_global_default(registry().with(l.with_writer(file))).unwrap();
    } else {
        tracing::subscriber::set_global_default(registry().with(l.with_writer(stdout))).unwrap();
    };
}

fn main() {
    // START.set(Instant::now()).unwrap();
    start();
    // f();
}

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    rom: Option<String>,

    #[arg(short, long)]
    dir: Option<String>,
}

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

fn controller() {
    let mut active_gamepad = None;

    let mut gilrs = Gilrs::new().unwrap();
    loop {
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }

        if active_gamepad.is_none() {
            println!("Couldn't detect an active gamepad");
        } else {
            if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
                match gamepad.axis_data(Axis::LeftStickX) {
                    None => {
                        println!("Couldn't read axis data");
                    }
                    Some(x) => {
                        println!("x: {}", x.value());
                        println!("");
                    }
                }
            }
        }
    }
    exit(0);
}

fn start() {
    // controller();


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

    // let logger_config = configure_log(&config, true /* remove */);
    // let handle = log4rs::init_config(logger_config).unwrap();

    START.set(Instant::now()).unwrap();

    configure_tracing(config.trace_to_file);

    warn!("This is a warning");
    info!("[Info] Logging");
    debug!("[Debug] Logging");

    // if true {
    if false {
        let path = "c:\\t\\speaker-events.txt".to_string();
        let path = "c:\\t\\pop.cycles.txt".to_string();
        let path = "c:\\t\\archon.cycles.txt".to_string();
        play_file_rodio(&path);
        exit(0);
    }

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
            Some(receiver2), disks, Box::new(emulator_config.clone()));
        apple2.cpu.run();
    } else {
        let sender4 = sender2.clone();
        let config_file_minifb = config_file.clone();

        //
        // Spawn the speaker thread
        //
        let _ = thread::Builder::new().name("Maple // - Speaker".to_string()).spawn(move || {
            Speaker::new().run();
        });

        //
        // Spawn the controller thread
        //
        let _ = thread::Builder::new().name("Maple // - Controller".to_string()).spawn(move || {
            Joystick::default().run();
        });

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
                    Some(receiver2.clone()), disks.clone(), Box::new(ecm));
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
        if false {
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
                                info!("Change: {event:?}")
                            }
                        },
                        Err(error) => error!("Error: {error:?}"),
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
    config: Box<EmulatorConfigMsg>)
-> Apple2
{
    let di0 = config.config_file.hard_drive_1().map(|s| DiskInfo::n(&s));
    let di1 = config.config_file.hard_drive_2().map(|s| DiskInfo::n(&s));

    let mut m = Apple2Memory::new(disk_infos, [di0, di1], sender.clone());

    m.load_roms(config.config_file.rom_type());

    let mut cpu = AppleCpu::new(Cpu::new(m, logging_sender, config.config.clone()),
        config.clone(), sender.clone(), receiver);
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
