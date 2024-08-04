use iced::{Element, Length, Renderer, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, Column, container, Container, row, Space, text};
use crate::disk::disk_info::DiskInfo;
use crate::ui::iced::main_window::MainWindow;
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::message::InternalUiMessage::Eject;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::style::{m_group, MColor};

impl MainWindow {
    pub fn drives_window(&self) -> Container<InternalUiMessage> {
        container(if self.show_drives { self.floppies() } else { self.hard_drives() })
    }

    fn hard_drives(&self) -> Element<InternalUiMessage> {
        let drive1 = m_group("Hard drive 1".into(),
            Self::ts(true, 0, true, 0.0, 0, Shared::get_block_number(0), &Shared::get_hard_drive(0)));
        let drive2 = m_group("Hard drive 2".into(),
            Self::ts(true, 1, false, 0.0, 0, Shared::get_block_number(1), &Shared::get_hard_drive(1)));

        container(row![drive1, drive2])
            .into()
    }

    fn floppies(&self) -> Element<InternalUiMessage> {
        let drive1 = m_group("Drive 1".into(),
            Self::ts(false, 0, self.selected_drive == 0, Shared::get_track(0) as f32,
            Shared::get_sector(0), 0, &Shared::get_drive(0)));

        let drive2 = m_group("Drive 2".into(),
            Self::ts(false, 1, self.selected_drive == 1, Shared::get_track(0) as f32,
                Shared::get_sector(1), 0, &Shared::get_drive(1)));

        container(row![drive1, drive2])
            .into()
    }

    /// Display the Track # | Sector # for drives
    fn ts<'a>(is_hard_drive: bool, drive_number: usize, is_selected: bool, track: f32, sector: u8,
        block_number: u16,
        disk_info: &Option<DiskInfo>)
    -> Element<'a, InternalUiMessage, Theme, Renderer>
    {
        let disk_name = if let Some(di) = disk_info {
            di.name()
        } else {
            ""
        };

        let mut disk_name = text(disk_name.to_string());
        if is_selected {
            disk_name = disk_name.color(MColor::orange());
        }
        let track_sector = if is_hard_drive {
            container(Column::new()
                .push(container(row![
                    text("Block").color(MColor::gray0()), Space::with_width(5.0)]))
                .push(container(row![
                    text(format!("{block_number: >05}")).color(MColor::green1()),
                        Space::with_width(5.0)]))
            ).style(|_: &Theme| {
                container::Style {
                    // background: Some(Background::Color(MColor::blue1())),
                    ..Default::default()
                }
            })
        } else {
            container(Column::new()
                .push(container(row![
                    text("T").color(MColor::gray0()), Space::with_width(5.0),
                    text(format!("{track:02}")).color(MColor::green1())]))
                .push(container(row![
                    text("S").color(MColor::gray0()), Space::with_width(5.0),
                    text(format!("{sector:02}")).color(MColor::green1())]))
            ).style(|_: &Theme| {
                container::Style {
                    // background: Some(Background::Color(MColor::blue1())),
                    ..Default::default()
                }
            })
        };
        row![
            track_sector,
            container(disk_name)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Top)
                .width(Length::Fill),
            button(text("\u{23cf}")
                .size(20.0)
                .shaping(text::Shaping::Advanced))
            .style(button::text)
            .on_press(Eject(is_hard_drive, drive_number))
        ].spacing(5).into()
    }
}