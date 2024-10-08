use std::process::exit;
use std::time::Instant;

use crossbeam::channel::Sender;
use gilrs::{Axis, Button, Gilrs};
use iced::{Color, Element, keyboard, Length, Padding, Point, Rectangle, Renderer, Size, Theme};
use iced::keyboard::Key::Named;
use iced::mouse::Cursor;
use iced::widget::{container, Space};
use iced::widget::{Column, row, Row};
use iced::widget::button::danger;
use iced::widget::Canvas;
use iced::widget::canvas::{Cache, Event, event, Fill, Geometry, Path, Program};
use iced_aw::Tabs;

use crate::{InternalUiMessage, InternalUiMessage::*};
use crate::config_file::ConfigFile;
use crate::constants::{CPU_REFRESH_MS, HIRES_HEIGHT, HIRES_WIDTH, SAMPLE_RATE};
use crate::disk::drive::DriveStatus;
use crate::joystick::Joystick;
use crate::messages::{CpuDumpMsg, DrawCommand, SetMemoryMsg, ToCpu, ToMiniFb};
use crate::send_message;
use crate::speaker::Samples;
use crate::ui::hires_screen::{AColor, HiresScreen};
use crate::ui::iced::debug_tab::DebugTab;
use crate::ui::iced::disk_tab::DriveTab;
use crate::ui::iced::disks_tab::DisksTab;
use crate::ui::iced::keyboard::special_named_key;
use crate::ui::iced::message::SpecialKeyMsg;
use crate::ui::iced::nibbles_tab::NibblesTab;
use crate::ui::iced::shared::*;
use crate::ui::iced::style::{m_button, MColor};
use crate::ui::iced::tab::Tab;
use crate::ui::iced::ui_iced::{TabId, Window};


fn controller() {
    let mut gilrs = Gilrs::new().unwrap();

// Iterate over all connected gamepads
    for (_id, gamepad) in gilrs.gamepads() {
        println!("{} is {:?}", gamepad.name(), gamepad.power_info());
    }

    let mut active_gamepad = None;

    loop {
        // Examine new events
        while let Some(gilrs::Event { id, event, time }) = gilrs.next_event() {
            println!("{:?} New event from {}: {:?}", time, id, event);
            active_gamepad = Some(id);
        }

        // You can also use cached gamepad state
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            // println!("Found current gamepad: {:#?}", gamepad);
            if gamepad.is_pressed(Button::South) {
                println!("Button South is pressed (XBox - A, PS - X)");
            }
            let x = gamepad.value(Axis::RightStickX);
            if x > 0. {
                println!("RightStickX: {}", x);
            }
            // let gamepad_state = gamepad.state();
            // for (code, button_data) in gamepad_state.buttons() {
            //     println!("Code: {:#?} button_data: {:#?}", code, button_data);
            // }
        }
    }
    exit(0);
}


pub struct MainWindow {
    config_file: ConfigFile,
    last_update: Instant,
    sender: Option<Sender<ToCpu>>,
    sender_minifb: Option<Sender<ToMiniFb>>,
    /// 0 or 1
    pub selected_drive: usize,

    /// Tabs
    active_tab: TabId,
    disks_tab: DisksTab,
    nibbles_tab: NibblesTab,
    drive_tab: DriveTab,
    debug_tab: DebugTab,

    drive_statuses: [DriveStatus; 2],

    // Drawing
    cache: Cache,
    draw_commands: Vec<DrawCommand>,
    hires_screen: HiresScreen,

    samples: Samples,
    joystick: Joystick,
}

impl MainWindow {
    pub fn new(config_file: ConfigFile,
       sender: Option<Sender<ToCpu>>, sender_minifb: Option<Sender<ToMiniFb>>)
    -> Self {
        let mut result = Self {
            config_file: config_file.clone(),
            active_tab: TabId::DisksTab,
            drive_statuses: [ DriveStatus::default(), DriveStatus::default() ],
            draw_commands: vec![],
            last_update: Instant::now(),
            selected_drive: 0,
            sender, sender_minifb,

            disks_tab: Default::default(),
            nibbles_tab: Default::default(),
            drive_tab: Default::default(),
            debug_tab: Default::default(),
            cache: Default::default(),
            hires_screen: Default::default(),
            samples: Samples::default(),
            joystick: Joystick::default(),
        };
        result.disks_tab.update(Init(config_file.clone()));
        result.nibbles_tab.update(Init(config_file.clone()));

        result
    }

    fn update_context(&mut self) {
        let cpu = self.cpu();
        //
        // Draw commands
        //
        self.draw_commands = self.hires_screen.get_draw_commands(
            &cpu.memory, &cpu.aux_memory, self.config_file.magnification());
        // self.cache.clear();
        if !self.draw_commands.is_empty() {
            send_message!(&self.sender_minifb, ToMiniFb::Buffer(self.draw_commands.clone()));
        }

        //
        // Controller
        //
        // self.joystick.main_loop();

        //
        // Speaker
        //
        let cycles: Vec<u64> = Shared::get_speaker_events().iter().map(|e| e.cycle).collect();
        if ! cycles.is_empty() {
            let samples = self.samples.cycles_to_samples(cycles, SAMPLE_RATE);
            for s in samples {
                Shared::add_sound_sample(s);
            }
        }

        //
        // Speaker decay
        //
        // if let Some((i, s)) = Shared::get_last_sample_played() {
        //     if f32::abs(s) > 0.01 && i.elapsed().as_millis() > 250 {
        //         speaker_decay(s);
        //         Shared::set_last_sample_played(None);
        //     }
        // }
    }

    fn cpu(&self) -> CpuDumpMsg {
        Shared::get_cpu()
    }
}

impl Window for MainWindow {
    fn title(&self) -> String {
        "Main window".into()
    }

    fn view(&self) -> Element<InternalUiMessage> {
        // println!("Drawing main with id {id:#?}, opening debugger: {}", self.opening_debugger);
        let canvas = Canvas::new(self)
            .width(Length::Fixed((HIRES_WIDTH * self.config_file.magnification()).into()))
            .height(Length::Fixed((HIRES_HEIGHT * self.config_file.magnification()).into()));

        let buttons = container(Column::new()
            .push(m_button("Reboot", InternalUiMessage::Reboot).style(danger))
            .push(Space::with_height(15.0))
            .push(m_button("Debug", InternalUiMessage::OpenDebugger))
            .push(Space::with_height(15.0))
            .push(m_button("Swap", InternalUiMessage::Swap))
            .padding(Padding::from([0.0, 10.0, 0.0, 10.0]))
            .push(Space::with_height(5.0))
            .push(if Shared::get_show_drives() {
                m_button("HD", ShowHardDrives)
            } else {
                m_button("Drives", ShowDrives)
            })
        );

        let tabs: Element<'_, InternalUiMessage> = Tabs::new(InternalUiMessage::TabSelected)
            .push(TabId::DisksTab, self.disks_tab.tab_label(), self.disks_tab.view())
            .push(TabId::NibblesTab, self.nibbles_tab.tab_label(), self.nibbles_tab.view())
            .push(TabId::DriveTab, self.drive_tab.tab_label(), self.drive_tab.view())
            // .push(TabId::DebugTab, self.debug_tab.tab_label(), self.debug_tab.view())
            .set_active_tab(&self.active_tab)
            .height(Length::Fill)
            .into();
        // let t: Element<'b, InternalUiMessage> = tabs.into();

        // let tab: Tabs<InternalUiMessage, TabId>
        //     =             .into();
        // let container_tab = m_container(tab.into());
        let main_ui = Row::new()
            .push(buttons)
            .push(Column::new()
                .padding(0)
                .push(self.drives_window())
                .push(Space::with_height(10.0))
                .push(container(tabs).padding([0, 10, 0, 10]))
            )
            ;

        let row = row!(canvas, main_ui);
        row.into()
    }

    fn update(&mut self, message: InternalUiMessage) {
        match message {
            Tick => {
                // alog(&format!("Got {} draw commands", self.draw_commands.len()));
                if self.last_update.elapsed().as_millis() > CPU_REFRESH_MS {
                    self.last_update = Instant::now();
                    self.update_context();
                }
            }
            DiskInserted(is_hard_drive, drive, disk_info) => {
                if drive == 0 {
                    self.nibbles_tab.update(DiskInserted(is_hard_drive, drive, disk_info.clone()));
                }
                self.drive_tab.update2(DiskInserted(is_hard_drive, drive, disk_info));
            }
            TabSelected(selected) => {
                println!("Selected: {selected:#?}");
                self.active_tab = selected;
            }
            NewDirectorySelected(directory) => {
                println!("New directory selected: {directory:#?}");
                let mut dirs: Vec<String> = Vec::new();
                if let Some(d) = directory.clone() {
                    dirs.push(d);
                }
                self.config_file.set_disk_directories(dirs);
                self.disks_tab.update_directory(directory);
            }
            FilterUpdated(filter) => {
                self.disks_tab.update_filter(filter);
            }
            ClearFilter => {
                self.disks_tab.update(message);
            }
            PhaseSelected(phase_160) => {
                // Assume the nibbles tab is only showing the track map for drive 0
                Shared::set_phase_160(0, phase_160);
            }
            Reboot => {
                self.hires_screen.on_reboot();
                if let Some(sender) = &self.sender {
                    sender.send(ToCpu::Reboot).unwrap();
                }
            }
            DriveSelected(drive) => {
                self.selected_drive = drive;
            }
            ShowDrives => {
                self.config_file.set_show_hard_drive(false);
                Shared::set_show_drives(true);
            }
            ShowHardDrives => {
                self.config_file.set_show_hard_drive(true);
                Shared::set_show_drives(false);
            }
            Eject(is_hard_drive, drive_number) => {
                if is_hard_drive {
                    Shared::set_hard_drive(drive_number, None);
                } else {
                    Shared::set_drive(drive_number, None);
                }
                self.config_file.set_drive(is_hard_drive, drive_number, None);
            }
            FirstRead(_, _) | ClearDiskGraph => {
                self.drive_tab.update2(message);
            }
            _ => {
                println!("Unknown message {message:#?}");
            }
        }
    }
}

impl Program<InternalUiMessage> for MainWindow
{
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (event::Status, Option<InternalUiMessage>)
    {
        let mut pressed = true;
        let mut message: Option<SpecialKeyMsg> = None;
        // Handle ALT_LEFT = button joystick 0
        if let Event::Keyboard(keyboard::Event::KeyPressed{key: Named(nk), .. }) = &event {
            pressed = true;
            message = special_named_key(*nk);
        }
        // Handle ALT_RIGHT = button joystick 1
        if let Event::Keyboard(keyboard::Event::KeyReleased{key: Named(nk), ..}) = &event {
            pressed = false;
            message = special_named_key(*nk);
        }
        let result = message.map(|m| SpecialKey(m, pressed));

        (event::Status::Ignored, result)
    }

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle,
            _cursor: Cursor) -> Vec<Geometry<Renderer>>
    {
        if self.draw_commands.is_empty() {
            Vec::new()
        } else {
            // alog("draw()");
            self.cache.clear();
            let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
                let fill = Fill::from(Color::BLACK); // <AColor as Into<Color>>::into(max_color));
                let bg = Path::rectangle(bounds.position(), bounds.size());
                frame.fill(&bg, fill);

                for dc in &self.draw_commands {
                    match dc {
                        DrawCommand::Rectangle(x0, y0, x1, y1, color) => {
                            frame.fill_rectangle(Point::new(*x0, *y0),
                                                 Size::new(x1 - x0, y1 - y0),
                                                 Fill::from(<AColor as Into<Color>>::into(*color)))
                        }
                        _ => {
                            println!("Unknown message in draw: {:#?}", dc);
                        }
                    }
                }

                //
                // Scan lines
                //
                let w = bounds.width;
                for y in (0..bounds.height as u16).step_by(2) {
                    frame.fill_rectangle(Point::new(0.0, y as f32),
                        Size::new(w, 1.0),
                        Fill::from(MColor::black1()));
                }
            });

            vec![geometry]
        }
    }

}
