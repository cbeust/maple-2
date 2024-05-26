use eframe::egui::{Pos2, Ui};
use eframe::epaint::{Color32, Stroke};
use crate::disk::disk_controller::MAX_PHASE;
use crate::ui::ui::MyEguiApp;

const BG: Color32 = Color32::from_rgb(0x11, 0x11, 0x11);
const FG: Color32 = Color32::from_rgb(0xaa, 0xaa, 0xaa);

impl MyEguiApp {
    pub(crate) fn create_floppy_window(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let nwp = ui.next_widget_position();
            let a_s = ui.available_size_before_wrap();
            let center = Pos2::new(nwp.x + (a_s.x / 2.0), nwp.y + (a_s.y / 2.0));
            let size = a_s.x / 2.5;
            ui.painter().circle_filled(center, size, BG);

            let tracks = MAX_PHASE / 4;
            for i in 0..tracks {
                let this_size = size * i as f32 / tracks as f32;
                ui.painter().circle_stroke(center, this_size + 10.0,
                    Stroke::new(1.0, FG));
            }
            // Head position
            let phase = MAX_PHASE as f32 -  self.nibbles_selected_phase as f32;
            let y = center.y + phase * size / MAX_PHASE as f32;
            ui.painter().circle_filled(Pos2::new(center.x, y), 5.0, Color32::RED);

        });
    }
}