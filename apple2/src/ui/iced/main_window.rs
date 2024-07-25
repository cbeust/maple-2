use std::time::Instant;
use crossbeam::channel::Sender;
use iced::widget::{Canvas};
use iced::{Color, Element, keyboard, Length, Padding, Point, Rectangle, Renderer, Size, Theme};
use iced::keyboard::Key::Named;
use iced::widget::{container, Space};
use iced::widget::{row, Row, Column};
use iced::mouse::Cursor;
use iced::widget::button::danger;
use iced::widget::canvas::{Cache, Event, event, Fill, Geometry, Path, Program};
use iced_aw::Tabs;
use crate::config_file::ConfigFile;
use crate::ui::iced::tab::Tab;
use crate::ui::iced::style::{m_button};
use crate::constants::{CPU_REFRESH_MS, HIRES_HEIGHT, HIRES_WIDTH};
use crate::disk::drive::DriveStatus;
use crate::messages::{CpuDumpMsg, DrawCommand, SetMemoryMsg, ToCpu, ToMiniFb};
use crate::ui::hires_screen::{AColor, HiresScreen};
use crate::ui::iced::disks_tab::DisksTab;
use crate::ui::iced::nibbles_tab::NibblesTab;
use crate::ui::iced::ui_iced::{TabId, Window};
use crate::{send_message};
use crate::ui::iced::disk_tab::DiskTab;
use crate::ui::iced::keyboard::{special_named_key};
use crate::ui::iced::message::{SpecialKeyMsg};
use crate::ui::iced::shared::*;
use crate::{InternalUiMessage, InternalUiMessage::*};

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
    disk_tab: DiskTab,

    drive_statuses: [DriveStatus; 2],

    // Drawing
    cache: Cache,
    draw_commands: Vec<DrawCommand>,
    hires_screen: HiresScreen,

    // Drives window (true: show drives, false: show hard drives)
    pub show_drives: bool,
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
            disk_tab: Default::default(),
            cache: Default::default(),
            hires_screen: Default::default(),
            show_drives: Shared::hard_drive(0).is_none(),
        };
        result.disks_tab.update(Init(config_file.clone()));
        result.nibbles_tab.update(Init(config_file.clone()));

        result
    }

    fn update_context(&mut self) {
        // if matches!(cpu.run_status, RunStatus::Stop(_, _)) {
        //     println!("Received dump that pauses");
        // } else {
        //     println!("Received dump that continues");
        // }
        let cpu = self.cpu();
        // self.memory_tab.set_memory(cpu.memory.clone());
        // if self.is_paused() {
        //     self.debugger_tab.update(clone);
        // }
        // println!("  GUI reading memory: {}", cpu.memory.len());
        self.draw_commands = self.hires_screen.get_draw_commands(
            &cpu.memory, &cpu.aux_memory, self.config_file.magnification());
        // self.cache.clear();
        if !self.draw_commands.is_empty() {
            send_message!(&self.sender_minifb, ToMiniFb::Buffer(self.draw_commands.clone()));
        }
        // println!("Read {} draw commands", self.draw_commands.len());
        // self.cache.clear();
    }

    fn cpu(&self) -> CpuDumpMsg {
        Shared::cpu()
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
            .push(m_button("Drives", ShowDrives))
            .push(Space::with_height(5.0))
            .push(m_button("HD", ShowHardDrives))
        );

        let tabs: Element<'_, InternalUiMessage> = Tabs::new(InternalUiMessage::TabSelected)
            .push(TabId::DisksTab, self.disks_tab.tab_label(), self.disks_tab.view())
            .push(TabId::NibblesTab, self.nibbles_tab.tab_label(), self.nibbles_tab.view())
            // .push(TabId::DiskTab, self.disk_tab.tab_label(), self.disk_tab.view())
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
                self.disk_tab.update2(DiskInserted(is_hard_drive, drive, disk_info));
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
                println!("Showing drives");
                self.show_drives = true;
            }
            ShowHardDrives => {
                println!("Showing hard drives");
                self.show_drives = false;
            }
            Eject(is_hard_drive, drive_number) => {
                if is_hard_drive {
                    Shared::set_hard_drive(drive_number, None);
                } else {
                    Shared::set_drive(drive_number, None);
                }
                self.config_file.set_drive(is_hard_drive, drive_number, None);
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
            });

            vec![geometry]
        }
    }

}
