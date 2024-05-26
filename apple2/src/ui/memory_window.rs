use eframe::egui::{Color32, FontId, RichText, ScrollArea, Ui};
use crate::ui::ui::{MemoryTab, MyEguiApp};

impl MyEguiApp {
    fn memory_widget(ui: &mut Ui, memory: &Vec<u8>, address_start: u16, length: usize) {
        let address_start = address_start as usize;
        ScrollArea::vertical().show(ui, |ui| {
            let mut bytes_string: String = "".to_string();
            ui.vertical(|ui| {
                if ! memory.is_empty() {
                    let mut address = 0;
                    while address < length {
                        let address_string = format!("{:04X}", address + address_start);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(address_string).strong()
                                .font(FontId::monospace(14.0)));
                            for _ in 0..16 {
                                let byte = memory[address + address_start];
                                bytes_string.push_str(&format!("{:02X} ", byte));
                                address += 1;
                            }
                            ui.label(RichText::new(format!(": {}", bytes_string))
                                .font(FontId::monospace(14.0)));
                            bytes_string = "".to_string();
                        });
                    }
                }
            });
        });
    }

    pub fn create_memory_window(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                for (e, s) in [
                    (MemoryTab::Main, "Main"),
                    (MemoryTab::Text, "Text"),
                    (MemoryTab::TextAux, "TextAux"),
                    (MemoryTab::Graphic, "Graphic"),
                    (MemoryTab::GraphicAux, "Graphic Aux"),
                ] {
                    let rt = RichText::new(s).strong().color(Color32::YELLOW);
                    ui.selectable_value(&mut self.selected_memory_tab, e, rt);
                    ui.separator();
                }
            });
            match self.selected_memory_tab {
                MemoryTab::Main => {
                    ui.horizontal(|ui| {
                        ui.label("Memory:");
                        if ui.text_edit_singleline(&mut self.memory_address).changed() {
                            println!("New value: {}", self.memory_address);
                        }
                    });
                    ui.separator();

                    Self::memory_widget(ui, &self.cpu.memory, 0, 0x10000);
                }
                MemoryTab::Text => {
                    Self::memory_widget(ui, &self.cpu.memory, 0x400, 0x800);
                }
                MemoryTab::TextAux => {
                    Self::memory_widget(ui, &self.cpu.aux_memory, 0x400, 0x800);
                }
                MemoryTab::Graphic => {
                    Self::memory_widget(ui, &self.cpu.memory, 0x2000, 0x4000);
                }
                MemoryTab::GraphicAux => {
                    Self::memory_widget(ui, &self.cpu.aux_memory, 0x2000, 0x4000);
                }
            };
        });
    }
}