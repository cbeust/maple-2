use eframe::egui::{Align, Button, Color32, FontId, FontSelection, Frame, InnerResponse, Label, Layout, Margin, Response, RichText, ScrollArea, Stroke, Style, TextStyle, Ui, Vec2};
use eframe::egui::FontFamily::Monospace;
use eframe::egui::text::LayoutJob;
use cpu::operand::Operand;
use cpu::constants::{OPERANDS_6502, OPERANDS_65C02};
use cpu::disassembly::{Disassemble, DisassemblyLine};
use crate::messages::CpuStateMsg::{Paused, Running};
use crate::messages::ToCpu::CpuState;
use crate::ui::debugger_window::RowType::Highlighted;
use crate::ui::ui::MyEguiApp;

// https://en.wikipedia.org/wiki/Media_control_symbols
const PLAY: &str = "\u{23f5}";
const PAUSE: &str = "\u{23f8}";
const STEP: &str = "\u{23e9}";

#[derive(Clone, Copy, PartialEq)]
enum RowType {
    Even,
    Odd,
    Pc,
    Highlighted,
}

impl MyEguiApp {
    fn set_debugger_paused(&mut self, paused: bool) {
        self.debugger_emulator_paused = paused;
        self.sender.send(CpuState(if paused { Paused } else { Running })).unwrap();
    }

    fn operands(&self) -> &[Operand] {
        if self.config.config.is_65c02 { &OPERANDS_65C02 } else { &OPERANDS_6502 }
    }

    pub fn create_debugger_window(&mut self, ui: &mut Ui) {
        ui.set_min_width(self.min_width);

        ui.horizontal(|ui| {
            //
            // Only display the rest of the UI if the emulator is paused
            //
            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
            if self.debugger_emulator_paused {
                self.display_assembly_panel(ui);
                self.display_registers(ui);
            };
            });

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            self.display_play_buttons(ui);
        });
        });
    }

    /// Play buttons
    fn display_play_buttons(&mut self, ui: &mut Ui) {
        let size = 40.0;
        ui.horizontal(|ui| {
                if self.debugger_emulator_paused {
                    let button = ui.add_sized([size, size], Button::new(PLAY));
                    if button.clicked() {
                        self.set_debugger_paused(false);
                    }
                } else {
                    let button = ui.add_sized([size, size], Button::new(PAUSE));
                    if button.clicked() {
                        self.set_debugger_paused(true);
                    }
                }
                // ui.add_enabled(self.debugger_emulator_paused, Button::new(STEP));
            });
    }

    /// Registers
    fn display_registers(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            MyEguiApp::reg(ui, "PC", &mut self.debugger_pc);
            MyEguiApp::reg(ui, " A", &mut self.debugger_a);
            MyEguiApp::reg(ui, " X", &mut self.debugger_x);
            MyEguiApp::reg(ui, " Y", &mut self.debugger_y);
        });
    }

    /// Assembly listing window
    fn display_assembly_panel(&mut self, ui: &mut Ui) {
        ui.set_width(600.0);
        ui.set_height(300.0);
        ui.horizontal(|ui| {
            let mut highlighted_line: Option<u8> = None;
            ui.vertical(|ui| {
                let pc = self.cpu.pc.wrapping_sub(10);
                let lines = Disassemble::disassemble_multiple(&self.cpu.memory, self.operands(), pc, 30);
                let highlighted = if let Some(h) = self.debugger_current_line { h } else {
                    let address = u16::from_str_radix(&self.debugger_pc, 16).unwrap();
                    self.debugger_current_line = Some(address);
                    address
                };
                for (row, line) in lines.iter().enumerate() {
                    if Self::add_assembly_line(ui, row, line, self.cpu.pc, highlighted) {
                        highlighted_line = Some(line.operand_size);
                    }
                }
            });
            let size = 30.0;
            let address = self.debugger_current_line.unwrap_or(self.cpu.pc);
            let op = self.operands()[self.cpu.memory[address as usize] as usize];
            ui.vertical(|ui| {
                ui.style_mut().text_styles.insert(TextStyle::Button, FontId::new(24.0, Monospace));
                ui.add_sized([size, size], Button::new("\u{21d1}"));
                if ui.add_sized([size, size], Button::new("↑")).clicked() {
                    self.debugger_current_line = self.debugger_current_line.map(|v| v.wrapping_sub(1))
                }
                if ui.add_sized([size, size], Button::new("↓")).clicked() {
                    self.debugger_current_line = self.debugger_current_line.map(|v| v.wrapping_add(op.size as u16));
                }
                ui.add_sized([size, size], Button::new("\u{21d3}"));
            })
        });
    }

    /// Return [true] if this line is currently highlighted.
    fn add_assembly_line(ui: &mut Ui, row: usize, line: &DisassemblyLine, pc: u16, highlighted: u16) -> bool {
        fn format(value: &str, color: Color32, layout_job: &mut LayoutJob, style: &Style,
                row_type: RowType) {
            use RowType::*;
            let bg =  match row_type {
                Even => { Color32::from_rgb(0x11, 0x11, 0x11) }
                Odd => { Color32::from_rgb(0x22, 0x22, 0x22) }
                Pc => {Color32::from_rgb(0x66, 0x66, 0x66) }
                Highlighted => { Color32::from_rgb(0x33, 0x33, 0x33) }
            };
            RichText::new(value)
                .color(color)
                .text_style(TextStyle::Monospace)
                .background_color(bg)
                .append_to(layout_job, style, FontSelection::Default, Align::LEFT);
        }

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

        let style = Style::default();
        let mut layout_job = LayoutJob::default();
        let even = (row % 2) == 0;
        let row_type = if line.pc == pc { RowType::Pc }
            else if line.pc == highlighted { Highlighted }
            else if even { RowType::Even }
            else { RowType::Odd };

        format(&format!("{:04X}: ", line.pc), Color32::WHITE, &mut layout_job, &style, row_type);
        format(&bytes(line.op.opcode, &line.bytes), Color32::YELLOW, &mut layout_job, &style, row_type);
        format(&format!("{:>8}", line.name), Color32::WHITE, &mut layout_job, &style, row_type);
        format(&format!(" {:<30}", line.value), Color32::GREEN, &mut layout_job, &style, row_type);

        let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));
        let _ = ui.add(Label::new(galley));

        row_type == Highlighted
    }
}

