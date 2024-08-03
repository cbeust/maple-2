use iced::widget::{Button, Column, stack, text};
use iced::{Border, Color, Element, Font, Length, Renderer, Theme};
use iced::alignment::Horizontal;
use iced::widget::{button, Container, container};
use crate::ui::iced::message::InternalUiMessage;

pub struct MColor;

impl MColor {
    pub fn black1() -> Color { Color::from_rgb(0.1, 0.1, 0.1) }
    pub fn black3() -> Color { Color::from_rgb(0.3, 0.3, 0.3) }

    pub fn gray0() -> Color { Color::from_rgb(0.4, 0.4, 0.4) }
    pub fn gray1() -> Color { Color::from_rgb(0.6, 0.6, 0.6) }
    pub fn gray2() -> Color { Color::from_rgb(0.8, 0.8, 0.8) }
    pub fn green() -> Color { Color::from_rgb(0.0, 1.0, 0.0) }
    pub fn green1() -> Color { Color::from_rgb(0.0, 0.8, 0.0) }

    pub fn orange() -> Color { Color::from_rgb8(0xff, 0x8c, 0x00) }
    pub fn orange2() -> Color { Color::from_rgb8(0xff, 0xd7, 0x00) }

    pub fn dark_gray() -> Color { Color::from_rgb8(96, 96, 96) }
    pub fn gray() -> Color { Color::from_rgb8(160, 160, 160) }
    pub fn light_gray() -> Color { Color::from_rgb8(220, 220, 220) }
    pub fn white() -> Color { Color::from_rgb8(255, 255, 255) }

    pub fn brown() -> Color { Color::from_rgb8(165, 42, 42) }
    pub fn dark_red() -> Color { Color::from_rgb8(0x8B, 0, 0) }
    pub fn red() -> Color { Color::from_rgb8(255, 0, 0) }
    pub fn light_red() -> Color { Color::from_rgb8(255, 128, 128) }

    pub fn yellow() -> Color { Color::from_rgb8(255, 255, 0) }
    pub fn yellow1() -> Color { Color::from_rgb8(255, 255, 0xE0) }
    pub fn yellow2() -> Color { Color::from_rgb8(255, 255, 0xD0) }
    pub fn yellow3() -> Color { Color::from_rgb8(255, 255, 0xC0) }
    pub fn khaki() -> Color { Color::from_rgb8(240, 230, 140) }

    pub fn dark_green() -> Color { Color::from_rgb8(0, 0x64, 0) }
    pub fn light_green() -> Color { Color::from_rgb8(0x90, 0xEE, 0x90) }

    pub fn blue() -> Color { Color::from_rgb8(0, 0, 0xff) }
    pub fn blue1() -> Color { Color::from_rgb8(0, 0, 0xE0) }
    pub fn blue2() -> Color { Color::from_rgb8(0xAD, 0xD8, 0xd0) }
    pub fn blue3() -> Color { Color::from_rgb8(0xaa, 0xd8, 0xe6) }

    pub fn gold() -> Color { Color::from_rgb8(255, 215, 0) }

    pub fn button_text() -> Color { Color::from_rgb(0.9, 0.9, 0.0) }
    pub fn button_background() -> Color { Color::from_rgb(0.9, 0.0, 0.0) }

}

pub struct MyButtonStyle;

// impl button::StyleSheet for MyButtonStyle {
//     type Style = Theme;
//     fn active(&self, _style: &Self::Style) -> button::Appearance {
//         Appearance {
//             background: Some(Background::Color(MColor::green())),
//             text_color: MColor::button_text(),
//             border: Border::with_radius(4.0),
//             ..Default::default()
//         }
//     }
//
//     fn hovered(&self, style: &Self::Style) -> Appearance {
//         let active = self.active(style);
//
//         Appearance {
//             background: Some(Background::Color(MColor::green1())),
//             shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
//             ..active
//         }
//     }
//
// }

/// Convenience function to specify background and foreground colors for a container
/// that contains some text.
// pub fn colors(foreground: Color, background: Color) -> container::Appearance {
//     container::Appearance {
//         text_color: Some(foreground),
//         background: Some(Background::Color(background)),
//         ..Default::default()
//     }
// }

pub fn m_group2<'a, T: 'a>(title: String, element: Element<'a, T>) -> Container<'a, T> {
    let bold_font = Font {
        weight: iced::font::Weight::Bold,
        ..Default::default()
    };

    let v = vec![
        m_container(element).into(),
        container(text(title)).into(),
    ];
    container(stack(v))
    // container(Column::new()
    //     .spacing(2)
    //     .padding(5.0)
    //     .push(text(title).size(12).color(MColor::orange()).font(bold_font))
    //     .push(m_container(element))
    // )
}

pub fn m_group(title: String, child: Element<InternalUiMessage>) -> Container<InternalUiMessage> {
    let bordered_container: Container<InternalUiMessage> = container(child)
        .style(|t: &Theme| {
            container::Style {
                background: Some(MColor::black1().into()),
                border: Border {
                    color: t.palette().primary,
                    radius: 5.0.into(),
                    width: 1.0,
                },
                ..container::transparent(t)
            }
        })
        .padding(10.0); // [5.0, 5.0, 100.0, 5.0]);
    let outer_container = container(bordered_container).padding(10.0);
    let title = container(
        container(text(title))
            .style(|t: &Theme| {
                container::Style {
                    background: Some(Color::from_rgba(0.1, 0.1, 0.1, 1.0).into()),
                    text_color: Some(MColor::blue3()),
                    ..container::transparent(t)
                }
            })
            .padding([0, 5])
    )
        .padding([0, 0, 0, 20]);

    container(iced::widget::stack([
        outer_container.into(),
        title.into()
    ]))
}

/// A container with a border
pub fn m_container<T>(element: Element<T>)
        -> Container<T, Theme, Renderer>
{
    container(element)
        .padding(5.0)
        .style(|_| container::Style::default().with_border(
                MColor::gray1(), 1.0)
        )
}

/// A bigger and round button
pub fn m_button(label: &str, message: InternalUiMessage) -> Button<InternalUiMessage> {
    let b = button(
        text(label)
            .horizontal_alignment(Horizontal::Center)
            .size(20.0)
            .width(Length::Fixed(75.0)))
        // TODO: restore the green/yellow buttons
        // .style(iced::theme::Button::Custom(Box::new(MyButtonStyle)))
        .on_press(message);

    let mut border = Border::default();
    border.width = 3.0;
    border.color = Color::WHITE;
    b
    // .padding(10.0)
    // .style(
    //     container::Appearance {
    //         text_color: Some(BUTTON_TEXT),
    //         // background: Some(iced::Background::Color(Color::from_rgb8(0.8, 0.0, 0.0))),
    //         border,
    //         ..Default::default()
    //     }
    // )
}

pub const UI_WIDTH: u16 = 700;

pub mod disks {
    use iced::{Length, Padding};

    pub const DRIVE_FONT_SIZE: f32 = 10.0;
    pub const DRIVE_BUTTON_HEIGHT: Length = Length::Fixed(15.0);
    // pub const DRIVE_BUTTON_WIDTH: Length = Length::Fixed(15.0);
    // pub const HARD_DRIVE_BUTTON_WIDTH: Length = Length::Fixed(60.0);
    pub const FONT_SIZE: f32 = 12.0;
    // pub const SPACING: f32 = 9.0;

    pub fn padding() -> Padding { Padding::from([5.0, 0.0, 0.0, 0.0]) }
}
