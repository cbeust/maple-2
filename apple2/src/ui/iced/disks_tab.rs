use iced::widget::*;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{text, row};
use std::path::Path;
use iced::{Alignment, Background, Color, Element, Font, Length, Theme};
use iced::widget::{button, column, container, Container, Row,};
use iced::widget::button::Status;
use ignore::{DirEntry, Walk};
use rfd::FileDialog;
use crate::config_file::ConfigFile;
use crate::constants::{BUGGY_DISKS, DISKS_SUFFIXES};
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::message::InternalUiMessage::{LoadDrive, LoadHardDrive};
use crate::ui::iced::shared::Shared;
use crate::ui::iced::tab::Tab;
use crate::ui::iced::style::{disks, MColor, m_group};
use crate::ui_log;

#[derive(Default)]
pub struct DisksTab {
    /// The directory
    directory: Option<String>,
    /// The filter
    filter: String,
    /// The content of the directory
    all_disks: Vec<DisplayedDisk>,
    /// The disks we are actually displaying (might be filtered)
    displayed_disks: Vec<DisplayedDisk>,
}

impl DisksTab {
    pub fn update(&mut self, message: InternalUiMessage) {
        match message {
            InternalUiMessage::ClearFilter => {
                self.filter = "".to_string();
                self.recalculate();
            }
            InternalUiMessage::Init(config_file) => {
                let directory: Option<String> = if config_file.disk_directories().is_empty() {
                    None
                } else {
                    Some(config_file.disk_directories()[0].clone())
                };
                self.update_directory(directory);
            }
            _ => {}
        }
    }

    pub fn update_directory(&mut self, directory: Option<String>) {
        self.directory = directory.clone();
        if let Some(d) = directory {
            self.all_disks = Self::read_disks_directories(&[d]);
        } else {
            self.all_disks.clear();
        }
        self.recalculate();
    }

    pub fn update_filter(&mut self, filter: String) {
        self.filter = filter;
        self.recalculate();
    }

    pub fn pick_directory(config_file: &ConfigFile) -> Option<String> {
        let d = &config_file.disk_directories();
        let dir = if !d.is_empty() && Path::new(&d[0]).exists() {
            &d[0]
        } else {
            ""
        };

        let result = FileDialog::new()
            .add_filter("Apple disk",
                &DISKS_SUFFIXES.iter().map(|s| s.to_string()).collect::<Vec<_>>())
            .set_directory(dir)
            .pick_folder();

        if let Some(path_buf) = result {
            let s = path_buf.to_str().unwrap();
            Some(s.to_string())
        } else {
            None
        }
    }

    /// The user changed some filtering so recalculate the list of disks we're showing.
    fn recalculate_disks(&mut self, directories: &[String]) {
        self.all_disks = Self::read_disks_directories(directories);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let disks = self.all_disks.clone();

        use rayon::prelude::*;
        self.displayed_disks = disks.par_iter().filter_map(|d| {
            if self.filter.is_empty() || d.file_name.to_lowercase().contains(&self.filter) {
                // println!("Including {d:#?}");
                Some(d.clone())
            } else {
                // println!("Excluding {d:#?}");
                None
            }
        }).collect::<Vec<DisplayedDisk>>();
    }

    fn read_disks_directories(directories: &[String]) -> Vec<DisplayedDisk> {
        let mut result: Vec<DisplayedDisk> = Vec::new();
        for path in directories.iter() {
            let builder = Walk::new(path).filter(|f| {
                if let Ok(de) = f {
                    let name = de.file_name().to_str().unwrap().to_lowercase();
                    let mut result = false;
                    for suffix in DISKS_SUFFIXES.clone().into_iter() {
                        if name.ends_with(&suffix) {
                            result = true;
                            break
                        }
                    }
                    result
                } else {
                    false
                }
            });
            for file in builder {
                if let Ok(f) = file {
                    result.push(DisplayedDisk::new(f));
                } else {
                    ui_log(&format!("Error in file {:?}", file));
                }
            }
        };

        result.sort_by(|a, b| a.file_name.to_lowercase().partial_cmp(
            &b.file_name.to_lowercase()).unwrap());
        result.dedup_by(|a, b| a.file_name == b.file_name);
        result
    }
}

fn drive_button_style(theme: &Theme, status: Status) -> button::Style {
    let mut style = button::primary(theme, status);
    // if status == Status::Active {
    //     style.background = Some(Background::Color(Color::from_rgb8(0x46, 0x46, 0x46)));
    // }
    style
}

/// The style of a button loadedin the drive
fn drive_button_style_loaded(theme: &Theme, status: Status) -> button::Style {
    let mut style = button::danger(theme, status);
    // if status == Status::Active {
    //     style.text_color = MColor::orange();
    // }
    style
}

fn drive_button(label: String, highlight: bool, message: InternalUiMessage)
    -> Element<'static, InternalUiMessage>
{
    let style = if highlight { drive_button_style_loaded } else { drive_button_style };
    container(button(text(label).font(Font::MONOSPACE)
            .horizontal_alignment(Horizontal::Center)
            .size(disks::DRIVE_FONT_SIZE)
            // .width(w)
            .height(disks::DRIVE_BUTTON_HEIGHT)
            .color(Color::from_rgb8(0xce, 0xce, 0xce)))
        .style(style)
        .on_press(message)
        )
        .padding(2)
    .into()
}

fn drive_buttons(disk: &DisplayedDisk, highlight: bool) -> Element<InternalUiMessage> {
    let path = disk.path.clone();
    if disk.file_name.ends_with("hdv") {
        row![
            container(drive_button("HD1".into(), highlight, LoadHardDrive(0, path.clone()))),
            container(drive_button("HD2".into(), highlight, LoadHardDrive(1, path))),
        ]
    } else {
        row![
            container(drive_button(" 1 ".into(), highlight, LoadDrive(0, path.clone()))),
            container(drive_button(" 2 ".into(), highlight, LoadDrive(1, path))),
        ]
    }.into()
}

impl Tab for DisksTab {
    type Message = InternalUiMessage;

    fn title(&self) -> String {
        String::from("Disks")
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let content = match &self.directory {
            Some(_) if ! self.all_disks.is_empty() => {
                // The disk names
                let buttons = self.displayed_disks.iter().map(|disk| {
                    let is_buggy = BUGGY_DISKS.iter().any(|s| {
                        disk.file_name.contains(s)
                    });
                    let disk_text: Container<InternalUiMessage> = container(
                        text(disk.file_name.clone()).color(
                            if is_buggy { MColor::red() } else { MColor::yellow() }
                        ).size(disks::FONT_SIZE)
                    );
                    let path_0 = Shared::get_drive(0).map_or("".to_string(), |d| d.path().into());
                    let path_1 = Shared::get_drive(1).map_or("".to_string(), |d| d.path().into());
                    let highlight = disk.path == path_0|| disk.path == path_1;
                    let buttons = drive_buttons(&disk, highlight);
                    Row::new()
                        .align_items(Alignment::Center)
                        .padding(disks::padding())
                        .push(container(buttons).width(Length::FillPortion(2)))
                        // .push(drive_button("1".into(), 0, disk))
                        // .push(drive_button("2".into(), 1, disk))
                        .push(disk_text.width(Length::FillPortion(10)))
                    .width(Length::Fill)
                    .into()
                });
                column![
                    // The "Filter" row
                    Row::new()
                        .spacing(10)
                        .padding(15)
                        .align_items(Alignment::Center)
                        .push(text("Filter:"))
                        .push(text_input("", &self.filter).width(Length::Fill)
                            .on_input(InternalUiMessage::FilterUpdated))
                        .push(button("X").on_press(InternalUiMessage::ClearFilter)),

                    // Vertical space
                    vertical_space().height(5),

                    // The disks
                    m_group("Disks".into(), scrollable(
                        Column::with_children(buttons).padding(2)
                    ).into())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    // .padding(5),
                ].into()
            }
            _ => {
                container(button(container(
                    text("Please select a directory containing Apple disks")
                        .size(25))
                    .padding(10))
                .on_press(InternalUiMessage::DisksDirectorySelected))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .into()
            }
        };

        content
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DisplayedDisk {
    pub file_name: String,
    pub path: String,
}

impl DisplayedDisk {
    fn new(d: DirEntry) -> DisplayedDisk {
        DisplayedDisk {
            file_name: d.file_name().to_str().unwrap().to_string(),
            path: d.path().to_str().unwrap().to_string(),
        }
    }
}
