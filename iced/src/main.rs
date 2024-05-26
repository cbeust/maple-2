mod theme;

use std::fmt::format;
use std::num::ParseIntError;
use std::time::Instant;
use time::OffsetDateTime;

use iced::button::{self, Button};
use iced::{Alignment, Application, Canvas, canvas, Color, Column, Command, Element, Font, executor, Length, Point, Rectangle, Renderer, Row, Rule, Scrollable, scrollable, Settings, Size, Subscription, Text, TextInput, Container, Padding, alignment, container, Theme, application};
use iced::canvas::{Fill, Stroke};
use iced::container::{Appearance, StyleSheet};
use iced::Font::Default;
use iced::mouse::Interaction;
use iced::text_input;
use iced::theme::palette;
use rand::Rng;
use crate::alignment::Horizontal;

use cpu::cpu::{Cpu, RunStatus};

use crate::canvas::{Cursor, Event, FillRule, Frame, Geometry, Path, Program};
use crate::canvas::event::Status;
use crate::EmulatorStatus::PAUSED;

pub fn main() -> iced::Result {
    EmulatorApp::run(Settings::default())
    // theme::ThemeApp::run(Settings::default())
}

const WINDOW_TITLE: &str = "6502 Emulator (iced)";

const APPLE_FONT: Font = Font::External {
    name: "Apple ][",
    bytes: include_bytes!("../../fonts/PrintChar21.ttf")
};

const MONOSPACE: Font = Font::External {
    name: "Monospace",
    bytes: include_bytes!("../../fonts/FiraMono-Regular.ttf")
};

#[derive(PartialEq, Debug)]
enum EmulatorStatus {
    RUNNING,
    PAUSED,
}

struct EmulatorApp {
    text_input_pc: String,
    text_input_a: String,
    text_input_x: String,
    text_input_y: String,
    page: String,
    state_pc: text_input::State,
    state_a: text_input::State,
    state_x: text_input::State,
    state_y: text_input::State,
    state_single_step: button::State,
    state_run: button::State,
    state_stop: button::State,
    state_disassembly: scrollable::State,
    state_page: text_input::State,
    state_page_incremented: button::State,
    state_page_decremented: button::State,
    cpu: Cpu,
    window_title: String,
    emulator_status: EmulatorStatus,
    last_run_timestamp: Instant,
}

impl std::default::Default for EmulatorApp {
    fn default() -> Self {
        Self {
            text_input_pc: "0".to_string(),
            text_input_a: "0".to_string(),
            text_input_x: "0".to_string(),
            text_input_y: "0".to_string(),
            page: "0".to_string(),
            state_pc: text_input::State::default(),
            state_a: text_input::State::default(),
            state_x: text_input::State::default(),
            state_y: text_input::State::default(),
            state_single_step: button::State::default(),
            state_run: button::State::default(),
            state_stop: button::State::default(),
            state_disassembly: scrollable::State::default(),
            state_page: text_input::State::default(),
            state_page_incremented: button::State::default(),
            state_page_decremented: button::State::default(),
            cpu: cpu::misc::create_cpu(0x400),
            window_title: WINDOW_TITLE.to_string(),
            emulator_status: PAUSED,
            last_run_timestamp: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    PageChanged(String),
    PageSubmitted,
    PageDecremented,
    PageIncremented,
    RegisterPCChanged(String),
    RegisterAChanged(String),
    RegisterXChanged(String),
    RegisterYChanged(String),
    RegisterPCSubmitted,
    RegisterASubmitted,
    RegisterXSubmitted,
    RegisterYSubmitted,
    Run,
    SingleStep,
    Stop,
    Tick(OffsetDateTime),
}

struct Emulator {}

// impl<Message> Program<Message> for Emulator {
    // fn draw(&self, bounds: Rectangle, _cursor: Cursor) -> Vec<Geometry> {
    //     // println!("Drawing canvas at {},{} {}-{}", bounds.x, bounds.y, bounds.width, bounds.height);
    //     let mut frame = Frame::new(bounds.size());
    //     let mut rnd = rand::thread_rng();
    //     let max_x = bounds.x + bounds.width;
    //     let max_y = bounds.y + bounds.height;
    //     let x = rnd.gen_range(bounds.x..max_x);
    //     let y = rnd.gen_range(bounds.y..max_y);
    //     println!("Drawing points at {},{}", x, y);
    //     let stroke = Stroke {
    //         color: Color::new(0.8, 0., 0., 0.9),
    //         width: 4.,
    //         ..Stroke::default()
    //     };
    //     frame.fill_rectangle(Point { x: bounds.x, y: bounds.y },
    //                          Size { width: bounds.width, height: bounds.height},
    //                         Fill { color: Color::BLACK, rule: FillRule::NonZero });
    //     frame.fill_rectangle(Point { x, y }, Size { width: 10., height: 10. },
    //                          Fill { color: Color::new(1., 0., 0., 1.), rule: FillRule::NonZero });
    //     vec![frame.into_geometry()]
    // }
// }

impl EmulatorApp {
    fn update_model(&mut self) {
        if self.emulator_status == PAUSED {
            self.text_input_pc = format!("{:04X}", self.cpu.pc);
            self.text_input_a = format!("{:02X}", self.cpu.a);
            self.text_input_x = format!("{:02X}", self.cpu.x);
            self.text_input_y = format!("{:02X}", self.cpu.y);
        }
    }
}

const WIDGET_HEIGHT: Length = Length::Units(50);
const TEXT_SIZE: u16 = 24;
const PADDING: Padding = Padding::new(5);

macro_rules! register {
    ( $row:expr, $label:expr, $state:expr, $input:expr, $message:expr, $submit_message:expr ) => {
        $row.align_items(Alignment::Center)
            .spacing(5)
            .height(WIDGET_HEIGHT)
            .push(Text::new(format!("{} $", $label)).size(TEXT_SIZE))
            .push(TextInput::new(
                    &mut $state,
                    $label,
                    &mut $input,
                    $message
            )
            .size(TEXT_SIZE).padding(PADDING).width(Length::Units(50))
            .on_submit($submit_message)
        )

    };
}

impl EmulatorApp {
    fn step(&mut self, total_cycles: u128, update_ui: bool) {
        let mut cycles_so_far = 0_u128;
        let mut stop = false;

        while ! stop {
            let pc = self.cpu.pc;
            let run_status = self.cpu.step(false);
            match run_status {
                RunStatus::Continue(run_cycles) => {
                    cycles_so_far += run_cycles as u128;
                    stop = cycles_so_far >= total_cycles;
                },
                RunStatus::Stop(_, _, _) => {
                    stop = true;
                },
            }
            // println!("PC before and after: {:04X} {:04x}", pc, self.cpu.pc);
            if pc == self.cpu.pc {
                self.emulator_status = PAUSED;
                stop = true;
            }
            self.update_model();
            // println!("{}", self.cpu.memory.disassemble_multiple(self.cpu.pc, 1).iter()
            //     .map(|l| l.line.clone())
            //     .collect::<Vec<String>>()
            //     .join("\n"));
        }
        // println!("Ran {} cycles", cycles_so_far);
    }
}

impl Application for EmulatorApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut result = Self::default();
        result.update_model();
        (result, Command::none())
    }

    fn title(&self) -> String {
        format!("{} - Cédric Beust", WINDOW_TITLE.to_string())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        // println!("update(): {:?}", message);
        match message {
            Message::SingleStep => {
                self.step(1, true);
            },
            Message::Tick(_) => {
                let millis = self.last_run_timestamp.elapsed().as_millis();
                self.step(millis * 10_000, false);
                self.last_run_timestamp = Instant::now();
            }
            Message::Run => {
                self.last_run_timestamp = Instant::now();
                self.emulator_status = EmulatorStatus::RUNNING;
            },
            Message::Stop => {
                self.emulator_status = EmulatorStatus::PAUSED;
            }
            Message::RegisterPCChanged(value) => {
                self.text_input_pc = value;
            },
            Message::RegisterAChanged(value) => {
                self.text_input_a = value;
            },
            Message::RegisterXChanged(value) => {
                self.text_input_x = value;
            },
            Message::RegisterYChanged(value) => {
                self.text_input_y = value;
            },
            Message::PageChanged(value) => {
                if let Ok(page) = u8::from_str_radix(self.page.as_str(), 16) {
                    self.page = format!("{:X}", page);
                }
            },
            Message::PageDecremented => {
                if let Ok(mut page) = u8::from_str_radix(self.page.as_str(), 16) {
                    let new_page = (if page == 0 { 0xff } else { page - 1 });
                    self.page = format!("{:X}", new_page);
                }
            },
            Message::PageIncremented => {
                if let Ok(mut page) = u8::from_str_radix(self.page.as_str(), 16) {
                    let new_page = (if page == 0xff { 0 } else { page + 1 });
                    self.page = format!("{:X}", new_page);
                }
            },
            Message::RegisterPCSubmitted => {
                if let Ok(parsed_value) = u16::from_str_radix(self.text_input_pc.as_str(), 16) {
                    self.cpu.pc = parsed_value as usize;
                } else {
                    self.text_input_pc = format!("{:02X}", self.cpu.pc);
                }
            }
            Message::RegisterASubmitted => {
                if let Ok(parsed_value) = u8::from_str_radix(self.text_input_a.as_str(), 16) {
                    self.cpu.a = parsed_value;
                } else {
                    self.text_input_a = format!("{:02X}", self.cpu.a);
                }
            }
            Message::RegisterXSubmitted => {
                if let Ok(parsed_value) = u8::from_str_radix(self.text_input_x.as_str(), 16) {
                    self.cpu.x = parsed_value;
                } else {
                    self.text_input_x = format!("{:02X}", self.cpu.x);
                }
            }
            Message::RegisterYSubmitted => {
                if let Ok(parsed_value) = u8::from_str_radix(self.text_input_y.as_str(), 16) {
                    self.cpu.y = parsed_value;
                } else {
                    self.text_input_y = format!("{:02X}", self.cpu.y);
                }
            }
            Message::PageSubmitted => {
                if let Ok(parsed_value) = u16::from_str_radix(self.page.as_str(), 16) {
                    println!("New page: {}", parsed_value)
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        //
        // Registers
        //
        let mut row_registers = Row::new()
            .padding([10, 10, 10, 10])
            .spacing(10);
        row_registers = register!(row_registers, "PC", self.state_pc, self.text_input_pc,
            Message::RegisterPCChanged, Message::RegisterPCSubmitted);
        row_registers = register!(row_registers, "A", self.state_a, self.text_input_a,
            Message::RegisterAChanged, Message::RegisterASubmitted);
        row_registers = register!(row_registers, "X", self.state_x, self.text_input_x,
            Message::RegisterXChanged, Message::RegisterXSubmitted);
        row_registers = register!(row_registers, "Y", self.state_y, self.text_input_y,
            Message::RegisterYChanged, Message::RegisterYSubmitted);

        //
        // Disassembly
        //
        let row_disassembly = self.cpu.memory.disassemble_multiple(self.cpu.pc, 50);
        let mut scrollable = Scrollable::new(&mut self.state_disassembly).width(Length::Units(400));
        // let font_source = iced_graphics::font::Source::new();
        // let font = font_source.load(&[Font::Family::SansSerif, Font::Family::Serif]);
        if self.emulator_status == PAUSED {
            for dl in row_disassembly.iter() {
                let mut text = Text::new(dl.line.clone()).size(12).font(APPLE_FONT);
                if dl.pc == self.cpu.pc {
                    text = text.style(Color::from_rgb(0., 0., 1.));
                }
                scrollable = scrollable.push(text);
            }
        }

        let bordered = iced::theme::Container::Custom(|_theme| {
            Appearance {
                border_width: 1.0,
                border_radius: 8.0,
                border_color: Color::from([0.4, 0.4, 0.4]),
                ..Appearance::default()
            }
        });

        //
        // Memory
        //

        // Page number
        let page_row = Row::new()
            .align_items(Alignment::Center)
            .padding(5)
            .spacing(5)
            .height(Length::Units(50))
            .push(Text::new("Page: $").size(TEXT_SIZE))
            .push(TextInput::new(&mut self.state_page, "page", &mut self.page, Message::PageChanged)
                    .size(TEXT_SIZE).padding(PADDING)
                    .width(Length::Units(50))
                .on_submit(Message::PageSubmitted))
            .push(Button::new(&mut self.state_page_decremented,
                              Text::new("-").horizontal_alignment(Horizontal::Center).size(30))
                .height(Length::Units(35))
                .width(Length::Units(50))
                .on_press(Message::PageDecremented))
            .push(Button::new(&mut self.state_page_incremented,
                              Text::new("+").horizontal_alignment(Horizontal::Center).size(30))
                .height(Length::Units(35))
                .width(Length::Units(50))
                .on_press(Message::PageIncremented));

        // Memory dump
        let mut memory_window = Column::new().push(page_row);
        let page_value = u8::from_str_radix(self.page.as_str(), 16).unwrap();
        let start: u16 = (page_value as u16) << 8;
        let end: u16 = std::cmp::min(0xffff, start as u32 + 0x200) as u16;
        for i in (start..end).step_by(16) {
            let mut row = Row::new().spacing(5);
            let mut line: Vec<String> = Vec::new();
            row = row.push(Text::new(format!("{:04X}: ", i)).font(MONOSPACE));
            for j in 0..16 {
                let text = Text::new(format!("{:02X}",
                        self.cpu.memory.get_no_listener((i + j).into())))
                    .font(MONOSPACE);
                row = row.push(text);
            }
            memory_window = memory_window.push(row);
        }

        let memory_container = Container::new(memory_window);

        let left_column = Container::new(Column::new()
            .spacing(5)
            .push(row_registers)
            .push(Row::new()
                .padding([10, 10, 10, 10])
                .spacing(10)
                .push(Button::new(&mut self.state_single_step,
                          Text::new("Single Step").horizontal_alignment(Horizontal::Center))
                    .width(Length::Units(100))
                    .on_press(Message::SingleStep))
                .push(Button::new(&mut self.state_run,
                          Text::new("Run").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::Run)
                    .width(Length::Units(100)))
                .push(Button::new(&mut self.state_stop,
                          Text::new("Stop").horizontal_alignment(Horizontal::Center))
                    .on_press(Message::Stop)
                    .width(Length::Units(100)))
            )
            .push(Row::new()
                .padding([10, 10, 10, 10])
                .push(scrollable))
        ).style(bordered);

        let column = if self.emulator_status == PAUSED {
            Column::new()
                .padding([0, 10, 10, 10])
                .push(memory_container)
        } else {
            Column::new()
        };

        let right_column = Container::new(column)
            .style(bordered);

        Container::new(Row::new()
            .padding(10)
            .spacing(20)
            .push(left_column)
            .push(right_column))
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match self.emulator_status {
            PAUSED => Subscription::none(),
            EmulatorStatus::RUNNING => {
                iced::time::every(std::time::Duration::from_millis(16)).map(|_| {
                    Message::Tick(
                        time::OffsetDateTime::now_local()
                            .unwrap_or_else(|_| time::OffsetDateTime::now_utc()),
                    )
                })
            }
        }
    }

    // fn subscription(&self) -> Subscription<Message> {
    //     iced::time::every(std::time::Duration::from_millis(500)).map(|_| {
    //         Message::Tick
    //     })
    // }
}
