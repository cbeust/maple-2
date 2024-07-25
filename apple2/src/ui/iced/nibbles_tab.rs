use iced::widget::horizontal_rule;
use iced::widget::{text, Text};
use iced::widget::scrollable;
use iced::widget::{container, Row};
use iced::{Alignment, Border, Color, Element, Font, Length, Renderer, Theme};
use iced::alignment::{Horizontal};
use iced::widget::{button, Column};
use iced::widget::button::Status;
use crate::disk::bit_stream::{AreaType, TrackType};
use crate::Disk;
use crate::disk::disk_info::DiskInfo;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::message::{InternalUiMessage, InternalUiMessage::*};
use crate::ui::iced::tab::Tab;
use crate::ui::iced::style::{m_container, m_group, MColor};

#[derive(Default)]
pub struct NibblesTab {
    current_disk: Option<Disk>,
}

/// Highlight the current phase
fn highlight(theme: &Theme, status: Status) -> button::Style {
    let mut style = button::text(theme, status);
    style.background = Some(MColor::blue().into());
    // style.border = Border::rounded(12.0).with_color(MColor::yellow()).with_width(1);
    style
}

/// track_type_to_label
fn tttl(track_type: &TrackType) -> Text<'static> {
    match track_type {
        TrackType::Standard => { text("\u{25cf}").color(MColor::green()) }
        TrackType::Nonstandard => { text("\u{25cf}").color(MColor::red()) } // text("\u{1F534}") }
        TrackType::Empty => { text(" ") }
    }
}

impl NibblesTab {
    pub fn update(&mut self, message: InternalUiMessage) {
        fn to_disk(di: Option<DiskInfo>) -> Option<Disk> {
            match Disk::new_with_disk_info(di) {
                Ok(d) => { Some(d) }
                Err(_) => { None }
            }
        }

        match message {
            Init(config_file) => {
                self.current_disk = to_disk(config_file.drive_1().map(|p| DiskInfo::n(&p)));
            }
            DiskInserted(_, _, disk_info) => {
                self.current_disk = to_disk(disk_info);
            }
            _ => {}
        }
    }
}

impl Tab for NibblesTab {
    type Message = InternalUiMessage;

    fn title(&self) -> String {
        String::from("Nibbles viewer")
    }

    fn content(&self) -> Element<'_, Self::Message> {
        fn phase_widget(text: Text<'static>, phase: usize, current_phase: u8)
            -> Element<'static, InternalUiMessage>
        {
            button(text.width(Length::Fixed(10.0))
                    .shaping(text::Shaping::Advanced)
                    .font(Font::MONOSPACE)
                    .size(10))
                .style(if phase as u8 == current_phase { highlight } else { button::text })
                .on_press(PhaseSelected(phase as u8)).into()
        }

        fn phase_row(row: u8, tracks: &[TrackType], current_phase: u8)
            -> Row<'static, InternalUiMessage>
        {
            let children = (0..80).step_by(4).map(|i| {
                let phase = (i + row * 80) as usize;
                let text = container(
                    Text::<Theme, Renderer>::new(format!("{:02}", phase / 4))
                        .font(Font::MONOSPACE)
                        .size(14)
                        )
                    .align_x(Horizontal::Center);
                let column = container(Column::new()
                    .align_items(Alignment::Center)
                    .push(text)
                    .push(horizontal_rule(1))
                    .push(phase_widget(tttl(&tracks[phase]), phase, current_phase))
                    .push(phase_widget(tttl(&tracks[phase + 1]), phase + 1, current_phase))
                    .push(phase_widget(tttl(&tracks[phase + 2]), phase + 2, current_phase))
                    .push(phase_widget(tttl(&tracks[phase + 3]), phase + 3, current_phase))
                );
                column.into()
            }).collect::<Vec<Element<InternalUiMessage>>>();
            Row::with_children(children).spacing(5)
        }

        //
        // Current track
        //
        let current_track_number = (Shared::phase_160(0) as f32) / 4.0;
        let label = format!("Current track: {:0.2}  (phase: {})",
            current_track_number, Shared::phase_160(0));
        let current_track = container(text(label).size(20.0))
            .width(Length::Fill)
            .align_x(Horizontal::Center);

        //
        // Track map
        //
        let disk = self.current_disk.clone();
        let t2 = if let Some(ref disk) = &disk {
            (0..160).map(|phase| {
                disk.analyze_track(phase).track_type
            }).collect::<Vec<TrackType>>()
        } else {
            vec![TrackType::Empty; 160]
        };

        let phase = Shared::phase_160(0);
        let track_map = m_group("Track map".into(),
            Column::new()
                .push(phase_row(0, &t2, phase))
                .push(phase_row(1, &t2, phase))
            .spacing(10)
            .padding(10)
            .into()
        );

        //
        // Nibbles
        //
        let ADDRESS_MARKER = MColor::yellow();
        let ADDRESS_CONTENT = MColor::white();
        let DATA_MARKER = MColor::orange();
        let DATA_CONTENT = MColor::white();
        let SYNC_BITS: Color = MColor::blue2();

        let mut column: Column<InternalUiMessage> = Column::new();
        if let Some(ref disk) = &disk {
            let mut address = 0_u16;
            let analyzed_track = disk.analyze_track(Shared::phase_160(0) as usize);
            let nibbles = &analyzed_track.nibbles;
            let mut index = 0_usize;
            while index < nibbles.len() {
                let mut row: Row<InternalUiMessage> = Row::new();
                let address_string = format!("{:04X}: ", address);
                row = row.push(text(address_string)
                    .font(Font::MONOSPACE)
                    .color(MColor::gray1())
                    .size(16));
                address += 16;

                let mut this_row = 0;
                while index < nibbles.len() && this_row < 16 {
                    let nibble = nibbles[index];
                    let n = format!("{:02X} ", nibble.value);
                    let color = match nibble.area_type {
                        AreaType::AddressPrologue => { ADDRESS_MARKER }
                        AreaType::AddressContent => { ADDRESS_CONTENT }
                        AreaType::AddressEpilogue => { ADDRESS_MARKER }
                        AreaType::DataPrologue => { DATA_MARKER }
                        AreaType::DataContent => { DATA_CONTENT }
                        AreaType::DataEpilogue => { DATA_MARKER }
                        _ => { Color::WHITE }
                        // AreaType::Unknown => {}
                    };
                    let nibble_text = container(text(n).font(Font::MONOSPACE).color(color).size(14));
                    let sb = if nibble.sync_bits >= 2 {
                        format!("{:02}", nibble.sync_bits)
                    } else {
                        "  ".to_string()
                    };
                    let sync_bits_text = container(text(sb).font(Font::MONOSPACE)
                        .color(SYNC_BITS)
                        .size(10));
                    row = row.push(iced::widget::column![nibble_text, sync_bits_text]);
                    index += 1;
                    this_row += 1;
                }

                column = column.push(row.spacing(2).width(Length::Fill)).spacing(5);
            }
        }
        let container = m_group("Nibbles".into(),
            scrollable(column).into())
                .width(Length::Fill)
                .height(Length::Fill);

        //
        // Finally, layout the tab
        //
        let content = iced::widget::column![
            current_track,
            track_map,
            container,
        ].spacing(10).
            into();

        content
    }
}
