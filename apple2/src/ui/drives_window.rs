use eframe::egui::{Color32, FontId, Label, Pos2, Response, RichText, Ui, Vec2};
use rfd::FileDialog;
use crate::disk::disk_info::DiskInfo;
use crate::disk::drive::DriveStatus;
use crate::messages::ToCpu::{LoadDisk, LockDisk, UnlockDisk};
use crate::send_message;
use crate::ui::ui::MyEguiApp;

impl MyEguiApp {
    pub fn create_drives_window(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for i in 0..2 {
                //
                // Track/Sector
                //
                ui.vertical(|ui| {
                    let track_sector = &self.track_sectors[i];
                    fn ts(ui: &mut Ui, label: &str, value: &str) {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(label).font(FontId::monospace(14.0)));
                            ui.label(RichText::new(&format!("{:02}", value))
                                .color(Color32::from_rgb(0xcc, 0xcc, 0xcc))
                                .font(FontId::monospace(14.0)));
                        });
                    }
                    ts(ui, "T", &format!("{:02}", track_sector.track));
                    ts(ui, "S", &format!("{:02}", track_sector.sector));
                });

                ui.separator();

                //
                // Disk name
                //

                // Drive light (red: on, yellow: spinning down)
                let color = match self.drive_statuses[i] {
                    DriveStatus::On => { Some(Color32::RED) }
                    DriveStatus::Off => { None }
                    DriveStatus::SpinningDown => { Some(Color32::YELLOW) }
                };
                if let Some(c) = color {
                    let r = ui.available_rect_before_wrap();
                    ui.painter().circle_filled(Pos2::new(r.min.x + 2.0, r.min.y + 20.0), 8.0, c);
                }

                // Disk name
                let mut disk_name_label = RichText::new("");
                let mut disk_side_label = RichText::new("");
                if let Some(di) = &self.disk_infos[i] {
                    let file_name = di.name();
                    if self.disk_index == i {
                        disk_name_label = RichText::new(file_name).color(Color32::GOLD).strong()
                    } else {
                        disk_name_label = RichText::new(file_name)
                    };
                    if let Some(side) = di.side() {
                        disk_side_label = RichText::new(side).font(FontId::proportional(12.0));
                    }
                }

                ui.vertical(|ui| {
                    ui.add_sized(Vec2::new(self.min_width / 2.0 - 95.0, 20.0),
                        Label::new(disk_name_label));
                    if ! disk_side_label.is_empty() {
                        ui.add_sized(Vec2::new(self.min_width / 2.0 - 95.0, 22.0),
                            Label::new(disk_side_label));
                    }
                });

                fn icon(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
                    ui.button(RichText::new(icon).size(22.0)).on_hover_text(tooltip)
                }

                //
                // Open disk and other buttons
                //
                ui.vertical(|ui| {
                    if icon(ui, "\u{1f4c2}", "Open...").clicked() {
                        if let Some(file) = FileDialog::new()
                                .add_filter("Apple disk", &["woz", "dsk"])
                                .pick_file() {
                            let disk_info = DiskInfo::n(file.to_str().unwrap());
                            self.sender.send(LoadDisk(i, disk_info)).unwrap();
                        }
                    }
                    if let Some(di) = &self.disk_infos[i] {
                        // Unlock
                        if di.is_write_protected {
                            if icon(ui, "\u{1f513}", "Unlock disk").clicked() {
                                self.sender.send(UnlockDisk(i)).unwrap();
                            }
                        } else {
                            // Lock
                            if icon(ui, "\u{1f512}", "Lock disk").clicked() {
                                self.sender.send(LockDisk(i)).unwrap();
                            }
                        }
                    }
                });

                if i == 0 { ui.separator(); }
            }
        });
    }

}