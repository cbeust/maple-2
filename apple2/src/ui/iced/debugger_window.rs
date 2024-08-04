use std::num::ParseIntError;
use iced::widget::{checkbox, column, scrollable, Text};
use iced::{Alignment, Color, Element, Font, Length};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, Column, container, Container, horizontal_rule, row, Row, text, text_input};
use cpu::constants::OPERANDS_6502;
use cpu::cpu::{RunStatus, StatusFlags};
use cpu::disassembly::{Disassemble, DisassemblyLine};
use crate::ui::iced::ui_iced::{Window};
use crate::{ui_log};
use crate::config_file::ConfigFile;
use crate::messages::CpuDumpMsg;
use crate::ui::iced::memory_view::{memory_view, MemoryType};
use crate::ui::iced::message::InternalUiMessage;
use crate::ui::iced::style::{m_container, m_group, MColor};
use crate::ui::iced::message::InternalUiMessage::*;
use crate::ui::iced::shared::Shared;

// https://en.wikipedia.org/wiki/Media_control_symbols
const PLAY: &str = "\u{25B6}";
const PAUSE: &str = "\u{23f8}";
const STEP: &str = "\u{25b7}";

struct Breakpoint {
    address: u16,
    enabled: bool,
}

impl Breakpoint {
    fn new(address: u16, enabled: bool) -> Self {
        Self { address, enabled }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum RowType {
    Even,
    Odd,
    Pc,
    Highlighted,
}

#[derive(Default)]
pub struct MemoryViewState {
    pub memory: Vec<u8>,
    pub location: String,
    pub memory_type: MemoryType,
}

#[derive(Default)]
pub struct DebuggerWindow {
    // The memory address currently highlighted. If none, defaults to PC
    debugger_current_line: Option<u16>,
    breakpoints: Vec<Breakpoint>,
    /// The value of the breakpoint in the input text
    breakpoint_value: String,

    memory_state: MemoryViewState,
}

impl DebuggerWindow {
    pub fn new(memory_state: MemoryViewState, config_file: ConfigFile) -> Self {
        let mut result = Self {
            memory_state,
            ..Default::default()
        };
        config_file.breakpoints().iter().for_each(|bp| {
            result.breakpoints.push(Breakpoint::new(bp.address, bp.enabled));
        });

        result
    }

    fn a(&self) -> String { format!("{:02X}", self.cpu().a) }
    fn x(&self) -> String { format!("{:02X}", self.cpu().x) }
    fn y(&self) -> String { format!("{:02X}", self.cpu().y) }
    fn pc(&self) -> String { format!("{:04X}", self.cpu().pc) }
    fn s(&self) -> String { format!("{:02X}", self.cpu().s) }
    fn flags(&self) -> StatusFlags { self.cpu().p }

    fn cpu(&self) -> CpuDumpMsg {
        Shared::get_cpu()
    }

    /// Display the registers: A, X, Y, ...
    fn registers_view(&self) -> Element<InternalUiMessage> {
        fn reg(label: String, value: String) -> Element<'static, InternalUiMessage> {
            let s = 16;
            let width = (value.len() * 16) as f32;
            row![
                text(label).size(s).font(Font::MONOSPACE),
                text_input("", &value).size(s).on_input(RegisterA).width(Length::Fixed(width))
                    .font(Font::MONOSPACE),
            ].spacing(5).align_items(Alignment::Center).into()
        }

        column![
            row![
                reg("A".into(), self.a()),
                reg("X".into(), self.x()),
                reg("Y".into(), self.y()),
                reg("PC".into(), self.pc()),
                reg("S".into(), self.s())
            ].spacing(10).padding(5).align_items(Alignment::Center),
        ].into()
    }

    fn flags_view(&self) -> Element<'static, InternalUiMessage> {
        fn flag(name: &str, value: bool) -> Element<InternalUiMessage> {
            let color = if value { MColor::yellow() } else { MColor::gray() };
            text(name).font(Font::MONOSPACE).color(color).size(20).into()
        }

        // let _gradient = Linear::new(Radians::from(0))
        //     .add_stop(0.0, Color::from_rgb(0.0, 0.0, 0.6))
        //     .add_stop(1.0, Color::from_rgb(0.0, 0.0, 0.9))
        //     .into();

        let flags = self.flags();
        container(row![
            flag("N", flags.n()),
            flag("V", flags.v()),
            flag("-", false),
            flag("B", flags.b()),
            flag("D", flags.d()),
            flag("I", flags.i()),
            flag("Z", flags.z()),
            flag("C", flags.c()),
        ].spacing(10))
            // .style(container::Appearance {
            //     background: Some(Background::Gradient(gradient)),
            //     ..Default::default()
            // })
            .into()
    }

    /// Display the breakpoints
    fn breakpoints_view(&self) -> Vec<Element<'static, InternalUiMessage>> {
        let mut result = Vec::new();
        let mut first = true;

        for bp in &self.breakpoints {
            if ! first {
                result.push(horizontal_rule(2).into());
            }
            result.push(row![
                text_input("", &format!("${:04X}", bp.address)).size(14).on_input(EditBreakPoint),
                checkbox("", bp.enabled).size(16.0),
                button(delete_icon()).style(button::danger).on_press({
                    DebuggerDeleteBreakpoint(bp.address)
                }),
            ].align_items(Alignment::Center).padding(5).spacing(10).width(Length::Fill).into());
            first = false;
        }
        result
    }

    /// Display the controls: pause/play, step, ...
    fn stack_view(&self) -> Container<InternalUiMessage> {
        let cpu = self.cpu();
        let memory = &cpu.memory;
        let current_stack = cpu.s as usize;
        let mut rows: Vec<Element<InternalUiMessage>> = Vec::new();
        let min = if current_stack > 8 { current_stack - 8 } else { 0 };
        let max = if current_stack > 0xf8 { current_stack } else { current_stack + 8 };
        for i in (min..max).rev() {
            let mut row: Row<InternalUiMessage> = Row::new();
            let mut container_row: Container<InternalUiMessage> = container(row
                .push(text(format!("{:02X}: ", i))
                    .font(Font::MONOSPACE)
                    .color(MColor::gray1())
                    .size(12))
                .push(text(format!("{:02X}", memory[0x100 + i]))
                    .font(Font::MONOSPACE)
                    .size(12)));
            if i == current_stack {
                container_row = container_row.style(move |_| {
                    container::Style::default().with_background(MColor::blue())
                })
            }
            rows.push(container_row.into());
        }
        let column = Column::with_children(rows);
        m_group("Stack".into(), scrollable(column)
                .width(Length::Fixed(100.0))
                .into())
    }

    /// Display the controls: pause/play, step, ...
    fn control_view(&self) -> Element<InternalUiMessage> {
        fn b<'a>(label: String, tt: String, message: InternalUiMessage) -> Element<'a, InternalUiMessage> {
            let size = 30;
            let result = button(text(label).size(size).shaping(text::Shaping::Advanced))
                .style(button::text)
                .on_press(message);
            result.into()
            // Tooltip::new(result, tt, Position::Bottom).into()
        }

        let (play_pause, message) = (PLAY, DebuggerPlay);

        // let s = iced::widget::Stack::new();
        let top = m_group("CPU".into(),
            row![
                column![
                    container(row![
                        b(play_pause.into(), "Resume".into(), message), //.explain(MColor::red()),
                        b(STEP.into(), "Step".into(), DebuggerStep), //.explain(MColor::red()),
                    ].align_items(Alignment::Center)).width(Length::Fill).align_x(Horizontal::Center),
                    horizontal_rule(2),
                    container(self.registers_view()).width(Length::Fill).align_x(Horizontal::Center),
                    horizontal_rule(2),
                    container(self.flags_view()).width(Length::Fill).align_x(Horizontal::Center), // .explain(MColor::red()),
                ],
            ].spacing(10).into())
            .width(Length::Fill);

        top.into()
    }

    /// Display the assembly listing
    /// Return the Element and the number of the currently highlighted line
    fn assembly_view(&self) -> (Element<'static, InternalUiMessage>, Option<u16>) {
        let mut debugger_current_line: Option<u16> = None;
        let mut highlighted_line: Option<u8> = None;
        let cpu = self.cpu();
        let pc = cpu.pc.wrapping_sub(0);
        if ! cpu.memory.is_empty() {
            let lines = Disassemble::disassemble_multiple(&self.cpu().memory, &OPERANDS_6502,
                                                          pc, 300);
            let highlighted = if let Some(h) = self.debugger_current_line { h } else {
                let address = self.cpu().pc;
                debugger_current_line = Some(address);
                address
            };
            let mut column = Column::new();
            for (row, line) in lines.iter().enumerate() {
                let (this_row, highlighted) = Self::add_assembly_line(row, line, self.cpu().pc,
                                                                      highlighted);
                if highlighted {
                    highlighted_line = Some(line.operand_size);
                }
                column = column.push(this_row);
            }
            (m_group("Disassembly".to_string(), column.into()).into(), debugger_current_line)
        } else {
            (text("").into(), None)
        }

    }

    /// Return [true] if this line is currently highlighted.
    fn add_assembly_line(row: usize, line: &DisassemblyLine, pc: u16, highlighted: u16)
                         -> (Element<'static, InternalUiMessage>, bool)
    {
        fn format<'a>(value: String, color: Color, row_type: RowType) -> Element<'a, InternalUiMessage> {
            use RowType::*;
            let bg = match row_type {
                Even => { Color::from_rgb8(0x11, 0x11, 0x11) }
                Odd => { Color::from_rgb8(0x22, 0x22, 0x22) }
                Pc => { Color::from_rgb8(0x66, 0x66, 0x66) }
                Highlighted => { Color::from_rgb8(0x33, 0x33, 0x33) }
            };
            container(text(value).font(Font::MONOSPACE).size(14).color(color))
                .style(move |_| {
                    container::Style::default().with_background(bg)
                })
                .into()
            // RichText::new(value)
            //     .color(color)
            //     .text_style(TextStyle::Monospace)
            //     .background_color(bg)
            //     .append_to(layout_job, style, FontSelection::Default, Align::LEFT);
        }

        let even = (row % 2) == 0;
        let row_type = if line.pc == pc { RowType::Pc }
        else if line.pc == highlighted { RowType::Highlighted }
        else if even { RowType::Even }
        else { RowType::Odd };

        fn bytes(op: u8, bytes: &Vec<u8>) -> String {
            match bytes.len() {
                0 => {
                    format!("{:02X}      ", op)
                }
                1 => {
                    format!("{:02X} {:02X}   ", op, bytes[0])
                }
                _ => {
                    format!("{:02X} {:02X} {:02X}", op, bytes[0], bytes[1])
                }
            }
        }

        let row = row![
            format(format!("{:04X}: ", line.pc), Color::WHITE, row_type),
            format(bytes(line.op.opcode, &line.bytes), MColor::yellow(), row_type),
            format(format!("{:>8}", line.name), Color::WHITE, row_type),
            format(format!(" {:<30}", line.value), MColor::green(), row_type),
        ];

        (row.into(), row_type == RowType::Highlighted)
    }
}

impl Window for DebuggerWindow {
    fn title(&self) -> String {
        "Debugger".into()
    }

    fn view(&self) -> Element<InternalUiMessage> {
        // let f = Font {
        //     family: Family::Name("FiraSans"),
        //     weight: iced::font::Weight::Normal,
        //     stretch: Stretch::Normal,
        //     style: FontStyle::Normal,
        // };

        let breakpoints = Column::new()
            .spacing(5)
            .push(m_group("Breakpoints".into(), Column::new()
                    .push(Row::new()
                        .align_items(Alignment::Center)
                        .padding(5)
                        .spacing(10)
                        .push(text_input("", &self.breakpoint_value).width(Length::Fixed(100.0))
                            .on_input(DebuggerBreakpointValue))
                        .push(button(text("Add").size(12))
                            .style(button::success)
                            .on_press(DebuggerAddBreakpoint(self.breakpoint_value.clone()))))
                    .push(horizontal_rule(1))
                    .push(Column::with_children(self.breakpoints_view()))
                    .spacing(5)
                    .into()
                ));

        let content = if matches!(self.cpu().run_status, RunStatus::Stop(_, _)) {
            let element: Element<InternalUiMessage> = row![
                {
                    let (assembly_view, line) = self.assembly_view();
                    assembly_view
                },
                Column::new()
                    .push(self.control_view())
                    .push(breakpoints)
                    .spacing(0)
                    .push(memory_view(&self.memory_state)),
                self.stack_view(),
            ]
                .spacing(0)
                .into();
            (element, Horizontal::Left, Vertical::Top)
        } else {
            let element: Element<'static, InternalUiMessage> = button(
                text("Start Debugger").size(25))
                .on_press(StartDebugger).into();
            (element, Horizontal::Center, Vertical::Center)
        };

        let (c, h, v) = content;
        let container = m_container(c.into())
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(h)
            .align_y(v)
            .padding(10);

        container.into()
    }

    fn update(&mut self, message: InternalUiMessage) {
        match message {
            DebuggerDeleteBreakpoint(address) => {
                if let Some(index) = self.breakpoints.iter().position(|bp| bp.address == address) {
                    self.breakpoints.remove(index);
                }
            }
            DebuggerBreakpointValue(bp) => {
                match u16::from_str_radix(&bp, 16) {
                    Ok(_) => {
                        self.breakpoint_value = bp;
                    }
                    _ => { ui_log(&format!("Invalid hex value: {bp}")); }
                }
            }
            DebuggerAddBreakpoint(_) => {
                // Breakpoint has been added, clear the value
                self.breakpoint_value = "".into();
            }
            DebuggerMemoryLocationChanged(location) => {
                println!("Current location: {location}");
                match u16::from_str_radix(&location, 16) {
                    Ok(l) if (0..=65_535).contains(&l) => {
                        self.memory_state.location = location;
                    }
                    _ => {
                        println!("Illegal HEX: {location}");
                        self.memory_state.location = "".to_string();
                    }
                };
            }
            DebuggerMemoryLocationSubmitted => {
                println!("Location submitted: {}", &self.memory_state.location);
            }
            DebuggerMemoryTypeSelected(memory_type) => {
                println!("Memory type: {}", memory_type);
                self.memory_state.memory = if memory_type == MemoryType::Main {
                    Shared::get_cpu().memory
                } else {
                    Shared::get_cpu().aux_memory
                };
                self.memory_state.memory_type = memory_type;
            }
            _ => {
                ui_log(&format!("Debugger Tab ignoring message: {message:#?}"));
            }
        }
    }
}


// Fonts
const ICONS: Font = Font::with_name("Iced-Todos-Icons");

fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(Horizontal::Center)
}

fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}
