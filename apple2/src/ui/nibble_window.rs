use eframe::egui::{Color32, FontId, RichText, ScrollArea, Ui, Vec2};
use eframe::egui::FontFamily::Monospace;
use eframe::egui::ImageData::Font;
use eframe::egui::scroll_area::ScrollAreaOutput;
use crate::disk::bit_stream::{BitStreams, TrackType};
use crate::ui::ui::{MyEguiApp, ui_log};
use crate::disk::bit_stream::AreaType;
use crate::disk::disk::Disk;
use crate::disk::disk_controller::{DiskController};

const ADDRESS_MARKER: Color32 = Color32::from_rgb(0x00, 0x33, 0x99);
const ADDRESS_MARKER_BG: Color32 = Color32::from_rgb(0xcc, 0x99, 0x33);
const ADDRESS_CONTENT: Color32 = Color32::from_rgb(0x00, 0x66, 0xff);
const DATA_MARKER: Color32 = ADDRESS_MARKER;
const DATA_MARKER_BG: Color32 = ADDRESS_MARKER_BG;
const DATA_CONTENT: Color32 = Color32::from_rgb(0x33, 0xcc, 0x33);
const SYNC_BIT_FG: Color32 = Color32::from_rgb(0xcc, 0x66, 0xff);

// use egui::{Color32, Frame, Rounding, Ui, Vec2, paint::Gradient, Shape};
//
// // Function to highlight nibble sequences
// fn highlight_nibble_sequence(ui: &mut Ui, nibbles: Vec<&str>, opening_sequence: (&str, Color32), closing_sequence: (&str, Color32), background_color: Color32) {
//     // Define the styles for the opening and closing sequences
//     let opening_frame = Frame {
//         fill: opening_sequence.1,
//         rounding: Rounding::same(50.0),
//         ..Default::default()
//     };
//     let closing_frame = Frame {
//         fill: closing_sequence.1,
//         rounding: Rounding::same(50.0),
//         ..Default::default()
//     };
//
//     // Create a gradient for the main sequence
//     let gradient = Gradient::linear(
//         [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)], // Coordinates should be adjusted according to the actual use case
//         [background_color, Color32::TRANSPARENT],    // Gradient colors
//     );
//
//     ui.horizontal(|ui| {
//         ui.add(egui::Label::new(opening_sequence.0).frame(opening_frame));
//
//         // Custom painting for the gradient background on the main sequence
//         let (rect, _response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
//         ui.painter().add(Shape::rect_filled(rect, 0.0, gradient));
//
//         for nibble in nibbles.iter() {
//             ui.label(nibble); // This will have the gradient background
//         }
//
//         ui.add(egui::Label::new(closing_sequence.0).frame(closing_frame));
//     });
// }

impl MyEguiApp {
    pub fn create_nibble_window(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if let Some(di) = &self.disk_infos[0] {
                if self.nibbles_path != di.path() {
                    let bytes = DiskController::file_to_bytes(0, di, &None);
                    if bytes.is_some() {
                        self.nibbles_disk = match Disk::new(&di.path(), false /* read bit_streams */,
                                None) {
                            Ok(disk) => {
                                self.nibbles_path = di.path().clone();
                                Some(disk)
                            }
                            Err(err) => {
                                ui_log(&format!("Couldn't load disk: {}", err));
                                None
                            }
                        }
                    } else {
                        ui.label(&format!("Couldn't load {}", di.path()));
                    };
                }

                match &self.nibbles_disk.clone() {
                    Some(disk) => {
                        self.nibble_window(ui, disk);
                    }
                    _ => {
                        println!("Received None bitstreams");
                    }
                }
            } else {
                ui.label("No disk inserted");
            }
        });
    }

    pub fn nibble_window(&mut self, ui: &mut Ui, disk: &Disk) -> ScrollAreaOutput<()> {
        use AreaType::*;
        //
        // Top of the window
        //

        //
        // Disk overview
        //
        let font = FontId::new(12.0, Monospace);
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    for j in 0..2 {
                        ui.horizontal(|ui| {
                            for x in 0..20 {
                                ui.vertical(|ui| {
                                    ui.label(&format!("{:02}", x + (20 * j)));
                                    for y in 0..4 {
                                        let phase = j * 80 + (x * 4 + y);
                                        let analyzed_track = disk.analyze_track(phase);
                                        let empty_track = RichText::new("\u{2022}");
                                        let standard_track = RichText::new("\u{23fa}")
                                            .color(Color32::GREEN);
                                        let non_standard_track = RichText::new("\u{23fa}")
                                            .color(Color32::YELLOW);
                                        let (label, tooltip) = match analyzed_track.track_type {
                                            TrackType::Standard => {
                                                (standard_track, "Standard track")
                                            }
                                            TrackType::Nonstandard => {
                                                (non_standard_track, "Nonstandard track")
                                            }
                                            TrackType::Empty => {
                                                (empty_track, "Empty track")
                                            }
                                        };
                                        let l = ui.selectable_label(self.nibbles_selected_phase == phase,
                                            label.font(font.clone()))
                                            .on_hover_text(&format!("Phase: {phase}\n{}", tooltip));
                                        if l.clicked() {
                                            self.nibbles_selected_phase = phase;
                                        }
                                    }
                                });
                            }
                        });
                        if j == 0 { ui.separator(); }
                    }
                })
            });
        });

        let label = &format!("Current track: {:0.2}  (phase: {})",
            (self.nibbles_selected_phase as f32) / 4.0, self.nibbles_selected_phase);
        ui.horizontal(|ui| {
            ui.add_space(200.0);
            ui.label(RichText::new(label).size(20.0).color(Color32::LIGHT_BLUE));
        });

        ui.group(|ui| {
            ScrollArea::vertical().show(ui, |ui| {
                use AreaType::*;
                let mut address = 0_u16;

                //
                // Nibbles view
                //
                let analyzed_track = disk.analyze_track(self.nibbles_selected_phase);
                ui.vertical(|ui| {
                    let nibbles = &analyzed_track.nibbles;
                    let mut index = 0_usize;
                    while index < nibbles.len() {
                        ui.horizontal(|ui| {
                            let address_string = format!("{:04X}: ", address);
                            address += 16;
                            ui.label(RichText::new(address_string).strong()
                                .font(FontId::monospace(14.0)));
                            let mut this_row = 0;
                            while index < nibbles.len() && this_row < 16 {
                                let nibble = nibbles[index];
                                let n = &format!("{:02X} ", nibble.value);
                                let (fg, bg) = match nibble.area_type {
                                    AddressPrologue => { (ADDRESS_MARKER, Some(ADDRESS_MARKER_BG)) }
                                    AddressContent => { (ADDRESS_CONTENT, None) }
                                    AddressEpilogue => { (ADDRESS_MARKER, Some(ADDRESS_MARKER_BG)) }
                                    DataPrologue => { (DATA_MARKER, Some(DATA_MARKER_BG)) } // ping
                                    DataContent => { (DATA_CONTENT, None) }
                                    DataEpilogue => { (DATA_MARKER, Some(DATA_MARKER_BG)) }
                                    Unknown => { (Color32::from_rgb(0x80, 0x80, 0x80), None) }
                                };
                                ui.vertical(|ui| {
                                    // println!("Spacing: {:?}", ui.style_mut().spacing);
                                    ui.style_mut().spacing.item_spacing = Vec2::new(8.0, -3.0);
                                    if let Some(bg) = bg {
                                        ui.label(RichText::new(n)
                                            .color(fg)
                                            .background_color(bg)
                                            .font(FontId::monospace(14.0)));
                                    } else {
                                        ui.label(RichText::new(n)
                                            .color(fg)
                                            .font(FontId::monospace(14.0)));
                                    }
                                    let sync = if nibble.sync_bits >= 2 {
                                        format!("{: ^3}", nibble.sync_bits)
                                    } else {
                                        "   ".to_string()
                                    };
                                    ui.label(RichText::new(sync)
                                        .color(SYNC_BIT_FG)
                                        .font(FontId::monospace(10.0)));
                                });
                                index += 1;
                                this_row += 1;
                            }
                        });
                    }
                });
            })
        }).inner
    }
}