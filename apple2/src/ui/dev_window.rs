use crossbeam::channel::Sender;
use eframe::egui::{Align, Frame, Layout, TextEdit, TextStyle, Ui, Vec2};
use crate::messages::ToCpu::{GenerateDisassembly, GetMemory, TraceStatus};
use crate::messages::{GenerateDisassemblyMsg, ToCpu, TraceStatusMsg};
use crate::messages::ToUi::RgbModeUpdate;
use crate::ui::soft_switches::{SOFT_SWITCHES, SoftSwitch};
use crate::ui::ui::MyEguiApp;

impl MyEguiApp {
    pub(crate) fn create_dev_window(&mut self, ui: &mut Ui) {
        //
        // Trace
        //
        if ui.checkbox(&mut self.trace, "Trace").clicked() {
            println!("Trace: {}", self.trace);
            self.sender.send(TraceStatus(TraceStatusMsg {
                debug_asm: Some(self.trace),
                trace_file: Some(self.trace),
                ..Default::default()
            })).unwrap();
        }
        ui.separator();

        //
        // Disassemble
        //
        ui.horizontal(|ui| {
            let max_width = 100.0;
            ui.label("Disassemble from: $");
            ui.add(TextEdit::singleline(&mut self.disassemble_from)
                .desired_width(max_width));
            ui.label("to: $");
            ui.add(TextEdit::singleline(&mut self.disassemble_to)
                .desired_width(max_width));
            ui.label("Into file");
            ui.add(TextEdit::singleline(&mut self.disassemble_to_file)
                .desired_width(300.0));
            if ui.button("Generate disassembly").clicked() {
                let from = u16::from_str_radix(&self.disassemble_from, 16);
                let to = u16::from_str_radix(&self.disassemble_to, 16);
                match (from, to) {
                    (Ok(f), Ok(t)) => {
                        self.sender.send(GenerateDisassembly(GenerateDisassemblyMsg {
                            from: f, to: t,
                            filename: self.disassemble_to_file.clone(),
                        })).unwrap();
                    }
                    _ => {
                        println!("Illegal address somewhere");
                    }
                }
            }
        });
        ui.separator();

        //
        // Soft switches
        //
        ui.horizontal(|ui| {
            ui.set_height(ui.available_height());
            ui.vertical(|ui| {
                Frame::group(ui.style()).show(ui, |ui| {
                    for i in 0..4 {
                        add_soft_switch(ui, &mut self.cpu.memory, &SOFT_SWITCHES[i], &self.sender);
                    }
                    ui.add_space(200.0);
                })
            });

            ui.vertical(|ui| {
                ui.group(|ui| {
                    for i in 4..SOFT_SWITCHES.len() {
                        add_soft_switch(ui, &mut self.cpu.memory, &SOFT_SWITCHES[i], &self.sender);
                    }
                    ui.add_space(ui.available_height());
                });
            });

            // Other dev controls
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.group(|ui| {
                    checkbox(ui, &mut self.scan_lines, "Scan lines", || {});
                    ui.add_space(ui.available_height());
                });
            });
        });
    }
}

fn checkbox<F>(ui: &mut Ui, field: &mut bool, label: &str, mut closure: F)
    where F: FnMut()
{
    ui.style_mut().spacing.item_spacing = Vec2::new(20.0, 20.0);
    ui.style_mut().override_text_style = Some(TextStyle::Heading);
    if ui.checkbox(field, label).clicked() {
        closure();
    }
}

fn add_soft_switch(ui: &mut Ui, memory: &mut Vec<u8>, ss: &SoftSwitch, sender: &Sender<ToCpu>) {
    if ! memory.is_empty() {
        checkbox(ui, &mut ss.is_set(memory), &ss.name.clone(), || {
            let address = if ss.is_set(memory) { ss.off } else { ss.on };
            sender.send(GetMemory(address)).unwrap();
            ss.flip(memory);
        });
    }
}

