use std::fmt::{Display, Formatter};
use iced::widget::{Column, horizontal_rule, horizontal_space, pick_list, text_input};
use iced::widget::{container, Row, scrollable, text};
use iced::{alignment, Color, Element, Font, Length,};
use crate::ui::iced::debugger_window::{MemoryViewState};
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::message::InternalUiMessage::{DebuggerMemoryLocationChanged, DebuggerMemoryLocationSubmitted};
use crate::ui::iced::style::MColor;

#[derive(Clone, Debug, PartialEq, Default)]
pub enum MemoryType {
    #[default]
    Main,
    Aux
}

impl Display for MemoryType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str(
            match self {
                MemoryType::Main => { "Main" }
                MemoryType::Aux => { "Aux" }
            }
        );
        Ok(())
    }
}

const ALL_MEMORY_TYPES: [MemoryType; 2] = [ MemoryType::Main, MemoryType::Aux ];

pub fn memory_view(state: &MemoryViewState) -> Element<'static, InternalUiMessage> {
    //
    // Header: Go to widget + memory selection
    //
    let header: Row<_> = Row::new()
        .padding(2)
        .align_items(alignment::Alignment::Center)
        .push(text("Go to:"))
        .push(horizontal_space().width(Length::Fixed(10.0)))
        .push(text_input("", &state.location.to_string()).width(Length::Fixed(50.0))
            .on_input(DebuggerMemoryLocationChanged)
            .on_submit(DebuggerMemoryLocationSubmitted))
        .push(horizontal_space().width(Length::Fixed(30.0)))
        .push(text("Memory type:"))
        .push(horizontal_space().width(Length::Fixed(10.0)))
        .push(pick_list(ALL_MEMORY_TYPES, Some(state.memory_type.clone()),
            InternalUiMessage::DebuggerMemoryTypeSelected))
        ;

    //
    // Memory container
    //
    let font_size = 13;
    let total = 0x1000;
    let mut index: usize = 0;
    let mut memory_content: Column<InternalUiMessage> = Column::new();
    let location = if let Ok(location) = usize::from_str_radix(&state.location, 16) {
        location
    } else {
        0
    };
    let mut address = location;

    if ! state.memory.is_empty() {
        while index < total {
            let mut row: Row<InternalUiMessage> = Row::new();
            let address_string = format!("{:04X}: ", address);
            row = row.push(text(address_string).font(Font::MONOSPACE)
                .color(MColor::gray1()).size(font_size));
            address += 16;

            let mut this_row = 0;
            while index < total && this_row < 16 {
                let nibble = state.memory[index.wrapping_add(location)];
                let n = format!("{:02X} ", nibble);
                let text = container(text(n).font(Font::MONOSPACE).color(Color::WHITE).size(font_size));
                row = row.push(text);
                index += 1;
                this_row += 1;
            }
          memory_content = memory_content.push(row.spacing(0).width(Length::Fill)).spacing(1);
        }
    }

    let all = Column::new()
        .push(header)
        .push(horizontal_rule(2))
        .push(scrollable(memory_content))
        .spacing(10);

    let container = crate::ui::iced::style::m_group("Memory".to_string(), all.into())
        .width(Length::Fill)
        .height(Length::Fill);

    container.into()
}