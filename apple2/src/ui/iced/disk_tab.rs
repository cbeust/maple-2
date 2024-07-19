use std::cmp::min;
use std::f32::consts::PI;
use iced::{Color, Element, Length, Rectangle, Renderer, Size, Theme};
use iced::mouse::Cursor;
use iced::widget::{Canvas};
use iced::widget::canvas::{Cache, Geometry, Program};
use crate::disk::disk::Disk;
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::tab::Tab;

#[derive(Default)]
pub struct DiskTab {
    cache: Cache,
}

impl DiskTab {
    fn disk(&self) -> Result<Disk, String> {
        Disk::new_with_disk_info(Shared::drive(0))
    }
}

impl DiskTab {
    pub fn update2(&mut self, message: InternalUiMessage) {
        match message {
            InternalUiMessage::DiskInserted(_, _, _) => {
                self.cache.clear();
            }
            _ => { println!("DiskTab ignoring message {message:#?}"); }
        }
    }
}

impl Tab for DiskTab {
    type Message = InternalUiMessage;

    fn title(&self) -> String {
        "Disk".into()
    }

    fn content(&self) -> Element<'_, InternalUiMessage> {
        let canvas = Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill);

        canvas.into()
    }
}

#[derive(Default)]
pub struct MyState;

impl<InternalUiMessage> Program<InternalUiMessage> for DiskTab {
    type State = MyState;

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle,
        _cursor: Cursor) -> Vec<Geometry<Renderer>>
    {
        fn bits_to_color(count: usize, bits: Vec<u8>) -> Color {
            let ones = bits.iter().filter(|b| **b == 1).count();
            if ones < count / 3 {
                Color::BLACK
            } else if ones >= count / 3 && count < 2 * count / 3 {
                Color::from_rgb(0.5, 0.5, 0.5)
            } else {
                Color::WHITE
            }
        }

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let radius = (min(bounds.size().width as u16, bounds.size().height as u16) - 300) as f32;

            let mut radius2 = radius;
            if let Ok(disk) = self.disk() {
                for i in 0..160 {
                    let mut index = 0;
                    let stream = disk.get_stream(i);
                    let diameter = 2.0 * radius2 * PI;
                    let ratio = stream.len() as f32 / diameter;
                    let mut angle = 0.0;
                    let increment = PI / stream.len() as f32 - 0.001;
                    while angle < PI * 2.0 && index < stream.len() {
                        let mut bits: Vec<u8> = Vec::new();
                        for i in 0..ratio as usize {
                            bits.push(stream.next_bit((index + i) % stream.len()));
                        }
                        angle += increment;
                        index += 1;
                        let x = frame.center().x + f32::cos(angle) * radius2;
                        let y = frame.center().y + f32::sin(angle) * radius2;
                        let color = bits_to_color(ratio as usize, bits);
                        frame.fill_rectangle([x, y].into(), Size::new(1.0, 1.0), color);
                    }
                    radius2 -= 6.0;
                }
            }
        });

        vec![geometry]
    }
}
