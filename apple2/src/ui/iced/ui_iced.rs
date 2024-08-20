use std::sync::{Arc, Mutex};

use crossbeam::channel::{Receiver, Sender};
use iced::{Color, Settings, Size, Subscription, Task, window};
use iced::{Element, Theme};
use iced::widget::text;
use iced::window::Position;

use cpu::cpu::{RunStatus, StopReason};

use crate::{DiskInfo, send_message, ui_log};
use crate::config_file::ConfigFile;
use crate::constants::{HIRES_HEIGHT, HIRES_WIDTH};
use crate::messages::{CpuStateMsg, SetMemoryMsg, ToCpu, ToMiniFb, ToUi};
use crate::messages::ToCpu::*;
use crate::ui::iced::message::InternalUiMessage::*;
use crate::ui::hires_screen::AColor;
use crate::ui::iced::debugger_window::{DebuggerWindow, MemoryViewState};
use crate::ui::iced::disks_tab::DisksTab;
use crate::ui::iced::keyboard;
use crate::ui::iced::main_window::MainWindow;
use crate::ui::iced::memory_view::MemoryType;
use crate::ui::iced::message::{InternalUiMessage, SpecialKeyMsg};
use crate::ui::iced::shared::Shared;
use crate::ui::iced::style::*;

type UiMessage = ToUi;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TabId {
    #[default]
    DisksTab,
    NibblesTab,
    DriveTab,
    DebugTab,
}

pub fn main_iced(sender: Option<Sender<ToCpu>>,
                 receiver: Receiver<ToUi>,
                 sender_minifb: Option<Sender<ToMiniFb>>,
                 config_file: ConfigFile) -> iced::Result
{
    let mut window_settings = window::Settings::default();
    let mag = config_file.magnification();
    let width = (HIRES_WIDTH * mag + UI_WIDTH) as f32;
    let height = (HIRES_HEIGHT * mag) as f32;
    window_settings.size = Size::new(width, height);

    // sender.clone().unwrap().send(ToCpu::SaveGraphics);

    iced::daemon(ATitle {}, EmulatorApp::update, EmulatorApp::view)
        .settings(Settings {
            default_text_size: 12.into(),
            ..Default::default()
        })
        .theme(|_, _| Theme::KanagawaWave)
        .style(|_, _| {
            iced::daemon::Appearance {
                background_color: MColor::black1(),
                text_color: MColor::white(),
            }
        })
        .subscription(EmulatorApp::subscription)
        .font(include_bytes!("../../../fonts/icons.ttf").as_slice())
        .load(move || {
            window::open(window_settings.clone()).map(MainWindowOpened)
        })
        // .window_size(Size::new(width, height))
        .run_with(move || {
            let mut result = EmulatorApp {
                receiver: Some(receiver.clone()),
                sender: sender.clone(),
                sender_minifb: sender_minifb.clone(),
                config_file: config_file.clone(),
                ..Default::default()
            };
            result
        })
}

struct ATitle {}

impl iced::daemon::Title<EmulatorApp> for ATitle {
    fn title(&self, state: &EmulatorApp, _window_id: window::Id) -> String {
        let running = &format!("Running at {:.02} Mhz", state.emulator_speed);
        format!("Maple 2 - Cpu: {}", if state.is_paused() { "Paused" } else { running })
    }
}

struct EmulatorApp {
    receiver: Option<Receiver<UiMessage>>,
    sender: Option<Sender<ToCpu>>,
    sender_minifb: Option<Sender<ToMiniFb>>,
    config_file: ConfigFile,
    debugger_window: Option<DebuggerWindow>,
    debugger_id: Option<window::Id>,
    main_window: Option<MainWindow>,
    main_id: Option<window::Id>,
    opening_debugger: bool,
    emulator_speed: f32,
    exit: bool,
}

pub trait Window {
    fn title(&self) -> String;
    fn view(&self) -> Element<InternalUiMessage>;
    fn update(&mut self, message: InternalUiMessage);
}

unsafe impl Send for EmulatorApp {}

impl Default for EmulatorApp {
    fn default() -> Self {
        Self {
            receiver: None,
            sender: None,
            debugger_window: None,
            debugger_id: None,
            main_window: None,
            main_id: None,
            sender_minifb: None,
            config_file: Default::default(),
            opening_debugger: false,
            emulator_speed: 0.0,
            exit: false,
        }
    }
}

impl EmulatorApp {
    fn is_paused(&self) -> bool {
        match Shared::get_cpu().run_status {
            RunStatus::Continue(_) => { false }
            RunStatus::Stop(_, _) => { true }
        }
    }

    fn memory_view_state(&self) -> MemoryViewState {
        MemoryViewState {
            memory: Shared::get_cpu().memory,
            location: "".into(),
            memory_type: MemoryType::Main
        }
    }

    fn update(&mut self, message: InternalUiMessage) -> Task<InternalUiMessage> {
        use InternalUiMessage::*;
        let mut result: Vec<Task<InternalUiMessage>> = Vec::new();

        let mut load_drive = |path: String, drive_index, is_hard_drive: bool| {
            let drive_type = if is_hard_drive { "hard ".to_string() } else { "".to_string() };
            ui_log(&format!("Loading {drive_type}drive {drive_index} with {path}"));
            let disk_info = DiskInfo::n(&path);
            if let Some(ref mut w) = &mut self.main_window {
                Shared::set_show_drives(! is_hard_drive);
            }
            send_message!(&self.sender, LoadDisk(is_hard_drive, drive_index, disk_info));
        };

        match message {
            SpecialKey(key, pressed) => {
                match key {
                    SpecialKeyMsg::AltLeft => {
                        Shared::set_controller_button_value(0, pressed);
                    }
                    SpecialKeyMsg::AltRight => {
                        Shared::set_controller_button_value(1, pressed);
                }
                    _ => {}
                };
            }
            Key(key) => {
                for i in 0..16 {
                send_message!(&self.sender, SetMemory(SetMemoryMsg {
                        address: 0xc000 + i,
                    bytes: vec![key],
                }));
            }
            }
            Exit => {
                self.exit = true;
            }
            NewDirectorySelected(_)
                | TabSelected(_)
                | FilterUpdated(_)
                | ClearFilter
                | PhaseSelected(_)
                | Tick
                | Reboot
                | DriveSelected(_)
                | ShowDrives
                | ShowHardDrives
                | Eject(_, _)
                | FirstRead(_, _)
                | ClearDiskGraph
                =>
            {
                if let Some(ref mut main_window) = &mut self.main_window {
                    main_window.update(message.clone());
                }
            }
            DiskInserted(is_hard_drive, drive_index, ref di) => {
                let di2 = di.clone();
                self.config_file.set_drive(is_hard_drive, drive_index, di2.map(|d| d.path.clone()));
                if let Some(ref mut main_window) = &mut self.main_window {
                    main_window.update(message.clone());
                }
            }
            BreakpointWasHit(_) => {
                if let Some(ref mut main_window) = &mut self.main_window {
                    main_window.update(StartDebugger);
                }
            }
            Swap => {
                if let Some(sender) = &self.sender {
                    sender.send(SwapDisks).unwrap();
                } else {
                    println!("No sender to send Swap to");
                }
            }
            OpenDebugger => {
                self.opening_debugger = true;
                let w = window::open(window::Settings {
                    position: Position::Centered,
                    ..window::Settings::default()
                });
                result.push(w.map(DebuggerWindowOpened));
            }
            DebuggerWindowOpened(id) => {
                self.debugger_window = Some(DebuggerWindow::new(
                    self.memory_view_state(),
                    self.config_file.clone()));
                self.debugger_id = Some(id);
                println!("Opening Debugger, id is {id:#?}");
                self.opening_debugger = false;
                result.push(Task::done(StartDebugger));
            }
            MainWindowOpened(id) => {
                println!("Opening main window");
                self.main_window = Some(MainWindow::new(
                    self.config_file.clone(), self.sender.clone(),
                    self.sender_minifb.clone()));
                self.main_id = Some(id);
            }
            DisksDirectorySelected => {
                let config_file = self.config_file.clone();
                result.push(Task::perform(
                    async move {
                        DisksTab::pick_directory(&config_file)
                    },
                    NewDirectorySelected));
            }
            LoadDrive(drive_index, path) => {
                load_drive(path, drive_index, false);
            }
            LoadHardDrive(drive_index, path) => {
                load_drive(path, drive_index, true);
            }
            RegisterA(a) => {
                println!("New value for A: {a}");
            }
            StartDebugger => {
                println!("UI received StartDebugger");
                Shared::set_run_status(RunStatus::Stop(StopReason::Ok, 0));
                send_message!(&self.sender, CpuState(CpuStateMsg::Paused));
            }
            DebuggerPlay => {
                send_message!(&self.sender, CpuState(CpuStateMsg::Running));
            }
            DebuggerStep => {
                send_message!(&self.sender, CpuState(CpuStateMsg::Step));
            }
            DebuggerDeleteBreakpoint(address) => {
                self.config_file.delete_breakpoint(address);
            }
            DebuggerBreakpointValue(_)
                | DebuggerMemoryTypeSelected(_)
                | DebuggerMemoryLocationSubmitted
                | DebuggerMemoryLocationChanged(_) => {
                if let Some(ref mut window) = &mut self.debugger_window {
                    window.update(message);
                }
            }
            DebuggerAddBreakpoint(v) => {
                self.config_file.add_breakpoint(v.clone());
                if let Some(ref mut window) = &mut self.debugger_window {
                    window.update(DebuggerAddBreakpoint(v));
                    window.update(Init(self.config_file.clone()));
                }
            }
            EmulatorSpeed(speed) => {
                self.emulator_speed = speed;
            }
            WindowClosed(id) => {
                if let Some(window_id) = &self.main_id {
                    if *window_id == id {
                        result.clear();
                        result.push(iced::exit::<InternalUiMessage>());
                        send_message!(&self.sender, CpuState(CpuStateMsg::Exit));
                    }
                }
            }
            Load | TabClosed(_) | Init(_) | DebuggerPause | EditBreakPoint(_) => {
                // ignored
            }
            // _ => {
            //     // not handled yet
            // }
        }

        // if Shared::breakpoint_was_hit() {
        //     println!("Starting a task to start the debugger");
        //     result.push(Task::done(OpenDebugger));
        // }

        Task::batch(result)
    }

    fn view(&self, window_id: window::Id) -> Element<InternalUiMessage> {
        match (self.debugger_id, &self.debugger_window)  {
            (Some(id), Some(window)) if id == window_id => {
                window.view()
            }
            _ => {
                if let Some(window) = &self.main_window {
                    window.view()
                } else {
                    text("").into()
                }
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<InternalUiMessage> {

        let stream = iced::subscription::unfold(42, self.receiver.clone(), |receiver| async {
            let result = tokio::task::spawn_blocking(|| {
                let mut result = None;
                while result.is_none() {
                    if let Ok(m) = receiver.clone().unwrap().recv() {
                        // Map the messages we're receiving from the Cpu and other processes
                        // into internal UI messages, which are then used to drive the GUI
                        match m {
                            ToUi::BreakpointWasHit(a) => {
                                println!("Mapping message ToUi -> Internal");
                                result = Some(OpenDebugger);
                            }
                            ToUi::DiskInserted(drive, di) => {
                                result = Some(DiskInserted(false, drive, di));
                            }
                            ToUi::HardDriveInserted(drive, di) => {
                                result = Some(DiskInserted(true, drive, di));
                            }
                            ToUi::EmulatorSpeed(speed) => {
                                result = Some(EmulatorSpeed(speed));
                            }
                            ToUi::DiskSelected(drive)=> {
                                result = Some(DriveSelected(drive));
                            }
                            ToUi::Exit => {
                                result = Some(Exit);
                            }
                            ToUi::Config(_) => {}
                            ToUi::KeyboardStrobe => {}
                            ToUi::DriveMotorStatus(_, _) => {}
                            ToUi::RgbModeUpdate(_) => {}
                            ToUi::FirstRead(drive, phase_160) => {
                                result = Some(FirstRead(drive, phase_160));
                            }
                        }
                    }
                }
                (result.unwrap(), receiver)
            }).await;
            
            match result {
                Ok(r) => { r }
                Err(_) => { panic!("Should not happen")}
            }
        });

        let mut subscriptions = vec![
            iced::keyboard::on_key_press(keyboard::handle_keyboard),
            iced::time::every(std::time::Duration::from_millis(100)) // CPU_REFRESH_MS as u64))
                .map(|_| Tick),
            window::close_events().map(WindowClosed),
            // stream,
        ];
        if ! self.exit && matches!(Shared::get_cpu().run_status, RunStatus::Continue(_)) {
            subscriptions.push(stream);
        }

        Subscription::batch(subscriptions)
    }

}

impl From<AColor> for Color {
    fn from(value: AColor) -> Self {
        let (r, g, b) = value.to_rgb();
        Color::from_rgb((r as f32) / 256., (g as f32) / 256., (b as f32) / 256.)
    }
}
