use iced::{Alignment, Background, Color, Element, Font, Length, Pixels};
use iced::widget::{Column, container, Row, row, Text, text, text_input};
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::style::MColor;
use crate::ui::iced::tab::Tab;

#[derive(Default)]
pub struct DebugTab {

}

const FONT_SIZE: u16 = 16;

fn label_text(label: String, value: String) -> Element<'static, InternalUiMessage> {
    let width = (value.len() * 16) as f32;
    let font_size: Pixels = FONT_SIZE.into();
    row![
        text(label).size(font_size).font(Font::MONOSPACE),
        text_input("", &value)
            .size(font_size)
            .style(|theme, status| {
                let mut result = text_input::default(theme, status);
                result
            })
            .width(Length::Fixed(width))
            .font(Font::MONOSPACE),
    ]
    .spacing(5)
    .align_items(Alignment::Center)
    .into()
}

impl Tab for DebugTab {
    type Message = InternalUiMessage;

    fn title(&self) -> String {
        "Debug".into()
    }

    fn content(&self) -> Element<'_, InternalUiMessage> {
        let values = Shared::get_controller_raw_values();
        let mut col = Column::new();
        for index in 0..2 {
            col = col.push(label_text(
                format!("Paddle {index}:"),
                format!("{:02X}", values[index])));
        }
        for index in 0..2 {
            let mut container = container(
                text(format!("Button {index}"))
                    .size(FONT_SIZE)
            );
            let pressed = Shared::get_controller_button_value(index);
            if pressed {
                container = container.style(|theme| {
                    let mut result = container::Style::default();
                    result.background = Some(Background::Color(MColor::yellow()));
                    result
                });
            }
            col = col.push(container);
        }
        col.into()
    }
}
