use std::cmp::min;
use std::f32::consts::PI;

use iced::{Alignment, Border, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use iced::mouse::Cursor;
use iced::widget::{button, Canvas, canvas, Column, container, Container, Row, text};
use iced::widget::canvas::{Cache, Geometry, Path, Program};
use once_cell::unsync::Lazy;

use crate::disk::disk::Disk;
use crate::disk::disk_controller::MAX_PHASE;
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::shared::Shared;
use crate::ui::iced::style::MColor;
use crate::ui::iced::tab::Tab;

struct HeadMovement {
    phase_160: u8,
}

#[derive(Default)]
pub struct DriveTab {
    cache: Cache,
    movements: Vec<HeadMovement>,
}

const COLOR_FULL_TRACK: Lazy<Color> = Lazy::new(|| Color::WHITE);
const COLOR_HALF_TRACK: Lazy<Color> = Lazy::new(|| MColor::orange());
const COLOR_QUARTER_TRACK: Lazy<Color> = Lazy::new(|| MColor::red());

impl DriveTab {
    fn disk(&self) -> Result<Disk, String> {
        Disk::new_with_disk_info(Shared::get_drive(0))
    }
}

/// Display the last 50 movements
const MOVEMENTS_MAX: usize = 40;

impl DriveTab {
    pub fn update2(&mut self, message: InternalUiMessage) {
        match message {
            InternalUiMessage::DiskInserted(_, _, _) => {
                self.cache.clear();
            }
            InternalUiMessage::FirstRead(_drive, phase_160) => {
                self.cache.clear();
                self.movements.push(HeadMovement { phase_160 });
            }
            InternalUiMessage::ClearDiskGraph => {
                self.movements.clear();
                self.cache.clear();
            }
            _ => { println!("DiskTab ignoring message {message:#?}"); }
        }
    }

    fn draw_graph(&self, bounds: Rectangle, renderer: &Renderer) -> Geometry {
        let margin = 15.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Background
            let background_bounds = bounds.size() - Size::new(margin * 2.0, margin * 2.0);
            frame.fill_rectangle(Point::new(margin, margin),
                background_bounds,
                MColor::black3());

            let width = bounds.width;
            let height = bounds.height;
            let scale_width = (width - margin * 2.0) / MOVEMENTS_MAX as f32;
            let scale_height = (height - margin * 2.0) / MAX_PHASE as f32;

            let phase_height = |phase: usize| {
                height - (phase as f32 * scale_height) - margin * 2.0
            };

            // Y axis (phases)
            let x = 0.0;
            for i in (0..MAX_PHASE).step_by(20) {
                let y = phase_height(i);
                // Track number
                frame.fill_text(canvas::Text {
                    content: format!("{:02}", i / 4),
                    position: Point::new(x, y - 5.0),
                    color: MColor::yellow(),
                    size: 10.0.into(),
                    ..Default::default()
                });
                // Horizontal line
                frame.fill_rectangle(Point::new(x, y), Size::new(width - margin * 2.0, 1.0),
                    MColor::blue());
            }

            let start = if self.movements.len() < MOVEMENTS_MAX { 0 }
                else { self.movements.len() - MOVEMENTS_MAX };

            // Display the last movements
            for i in 0..min(self.movements.len(), MOVEMENTS_MAX) {
                let m = &self.movements[i + start];
                let x = i as f32 * scale_width + margin;
                let y = phase_height(m.phase_160 as usize);
                let circle = Path::circle([x, y].into(), 3.0);
                let color = match m.phase_160 % 4 {
                    0 => { COLOR_FULL_TRACK }
                    2 => { COLOR_HALF_TRACK }
                    _ => { COLOR_QUARTER_TRACK }
                };
                frame.fill(&circle, *color);
                // frame.fill_rectangle(Point::new(x, y), Size::new(4.0, 4.0), Color::WHITE);

            }
        });

        geometry
    }

}

impl Tab for DriveTab {
    type Message = InternalUiMessage;

    fn title(&self) -> String {
        "Drive".into()
    }

    fn content(&self) -> Element<'_, InternalUiMessage> {
        fn c(title: &str, color: Color) -> Container<InternalUiMessage> {
            container(text(title).color(color).width(Length::Fill)).padding([0.0, 20.0, 0.0, 20.0])
        }
        let legend = container(
            Row::new()
                .push(c("Full track", *COLOR_FULL_TRACK))
                .push(c("Half track", *COLOR_HALF_TRACK))
                .push(c("Quarter track", *COLOR_QUARTER_TRACK))
                .spacing(10.0)
                .align_items(Alignment::Center)
        )
            .style(|_| {
                container::Style {
                    // background: Some(Background::from(MColor::black3())),
                    border: Border::default().with_radius(3.0).with_width(1.0).with_color(MColor::gray1()),
                    ..Default::default()
                }
            })
            .padding(5.0)
            ;

        Column::new()
            .push(Row::new()
                .push(container(text("Track\nnumber").color(MColor::yellow())))
                .push(legend)
                .push(button("Clear").on_press(InternalUiMessage::ClearDiskGraph))
                .spacing(10.0)
            ) // row
            .padding([5.0, 15.0, 0.0, 15.0])
        .push(Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Default)]
pub struct MyState;

impl<InternalUiMessage> Program<InternalUiMessage> for DriveTab {
    type State = MyState;

    fn draw(&self, _state: &Self::State, renderer: &Renderer, _theme: &Theme, bounds: Rectangle,
        _cursor: Cursor) -> Vec<Geometry<Renderer>>
    {
        let geometry = self.draw_graph(bounds, renderer);
        vec![geometry]
    }
}


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

fn draw_floppy(cache: &Cache, bounds: Rectangle, renderer: &Renderer, disk: &Result<Disk, String>)
    -> Geometry
{
    let geometry = cache.draw(renderer, bounds.size(), |frame| {
        let radius = (min(bounds.size().width as u16, bounds.size().height as u16) - 300) as f32;

        let mut radius2 = radius;
        if let Ok(disk) = disk {
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

    geometry

}

