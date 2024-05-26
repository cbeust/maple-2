use std::path::Path;
use eframe::egui::{Align, Color32, Label, Layout, RichText, ScrollArea, Ui, Vec2};
use ignore::Walk;
use rfd::FileDialog;
use crate::constants::{BUGGY_DISKS, DISKS_SUFFIXES};
use crate::disk::disk::Disk;
use crate::disk::disk_controller::{DiskController};
use crate::disk::disk_info::DiskInfo;
use crate::disk::disk_info::DiskType::{Dsk, Woz1, Woz2};
use crate::messages::ToCpu::LoadDisk;
use crate::ui::ui::{MyEguiApp, ui_log};

impl MyEguiApp {
    fn is_buggy(name: &str) -> bool {
        BUGGY_DISKS.iter().any(|d| {
            name.to_lowercase().contains(&d.to_lowercase().to_string())
        })
    }

    /// The user changed some filtering so recalculate the list of disks we're showing.
    fn recalculate_disks(&mut self) {
        let disks = self.disks_all_disks.clone();

        use rayon::prelude::*;
        self.disks_displayed_disks = disks.par_iter().filter_map(|d| {
            if self.disks_filter.is_empty() || d.path().to_lowercase().contains(&self.disks_filter) {
                #[allow(clippy::if_same_then_else)]
                if d.disk_type == Woz1 && self.disks_woz1 { Some(d.clone()) }
                else if d.disk_type == Woz2 && self.disks_woz2 { Some(d.clone()) }
                else if d.disk_type == Dsk && self.disks_dsk { Some(d.clone()) }
                else { None }
            } else {
                None
            }
        }).collect::<Vec<DiskInfo>>();
    }

    pub fn create_disks_window(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // ui.set_max_width(150.0);
            //
            // The list of all the disks
            //
            ui.vertical(|ui| {
                ScrollArea::vertical()
                        .auto_shrink(false)
                        .max_width(300.0)
                        .min_scrolled_height(600.0).show(ui, |ui| {
                    let disks = self.disks_displayed_disks.clone();
                    if ! disks.is_empty() {
                        for disk_info in &self.disks_displayed_disks {
                            ui.horizontal(|ui| {
                                //
                                // Drive 1 / Drive 2
                                //
                                let mut drive_label = |index: usize, label: &str| {
                                    let mut drive_label = RichText::new(label);
                                    if let Some(di) = &self.disk_infos[index] {
                                        if *di.path == disk_info.path {
                                            drive_label = RichText::new(label)
                                                .background_color(Color32::DARK_BLUE);
                                        }
                                    }

                                    if ui.button(drive_label).clicked() {
                                        let disk = DiskController::
                                            load_disk_new(&disk_info.path(), None);
                                        if let Ok(disk) = disk {
                                            self.sender.send(
                                                LoadDisk(index, disk.disk_info().clone())).unwrap();
                                        }
                                    };
                                };
                                drive_label(0, "Drive 1");
                                drive_label(1, "Drive 2");

                                //
                                // Name of the disk, in the correct color
                                //
                                let color = if Self::is_buggy(disk_info.name()) {
                                    Color32::RED
                                } else if disk_info.path().to_lowercase().ends_with("woz") {
                                    Color32::YELLOW
                                } else {
                                    Color32::LIGHT_BLUE
                                };
                                ui.scope(|ui| {
                                    ui.set_max_width(150.0);
                                    let max = std::cmp::min(60, disk_info.name().len());
                                    let n = &disk_info.name()[0..max];
                                    ui.add(Label::new(RichText::new(n).color(color)));
                                    // let r = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(300.0, 15.0));
                                    // ui.put(r, Label::new(RichText::new(disk_info.name()).color(color)));
                                });
                            });
                        }
                    } else if ui.button("Please select a directory containing disk images").clicked() {
                        self.select_directory();
                    }
                });
            });

            //
            // The UI on the right: filter, checkboxes
            //
            ui.vertical(|ui| {
                ui.group(|ui| {
                    ui.scope(|ui| {
                        ui.set_min_width(100.0);
                        ui.spacing_mut().item_spacing = Vec2::new(20.0, 20.0);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Open...").clicked() {
                                    self.select_directory();
                                }
                                if ui.button("Refresh").clicked() {
                                    self.disks_all_disks = MyEguiApp::read_disks_directories(
                                        self.config_file.disk_directories());
                                    self.recalculate_disks();
                                }
                            });
                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("Search: ");
                                if ui.text_edit_singleline(&mut self.disks_filter).changed() {
                                    self.recalculate_disks();
                                }
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("X").clicked() {
                                        self.disks_filter = "".to_string();
                                        self.recalculate_disks();
                                    }
                                });
                            });
                            if ui.checkbox(&mut self.disks_woz1, "Woz v1").clicked() {
                                self.recalculate_disks();
                            }
                            if ui.checkbox(&mut self.disks_woz2, "Woz v2").clicked() {
                                self.recalculate_disks();
                            }
                            if ui.checkbox(&mut self.disks_dsk, "Dsk").clicked() {
                                self.recalculate_disks();
                            }
                        });
                    });
                });
            });
        });
    }

    fn select_directory(&mut self) {
        let d = &self.config_file.disk_directories();
        let dir = if !d.is_empty() && Path::new(&d[0]).exists() {
            &d[0]
        } else {
            ""
        };
        if let Some(file) = FileDialog::new()
                .add_filter("Apple disk", &["woz", "dsk"])
                .set_directory(dir)
                .pick_folder() {
            let directories = vec![file.to_str().unwrap().to_string()];
            self.config_file.set_disk_directories(directories.clone());
            self.disks_all_disks = MyEguiApp::read_disks_directories(&directories);
            self.recalculate_disks();
        }
    }

    pub fn read_disks_directories(directories: &[String]) -> Vec<DiskInfo> {
        let mut result: Vec<DiskInfo> = Vec::new();
        for path in directories.iter() {
            let builder = Walk::new(path).filter(|f| {
                if let Ok(de) = f {
                    let name = de.file_name().to_str().unwrap().to_lowercase();
                    let mut result = false;
                    for suffix in DISKS_SUFFIXES.clone().into_iter() {
                        if name.ends_with(&suffix) {
                            result = true;
                            break
                        }
                    }
                    result
                } else {
                    false
                }
            });
            for file in builder {
                if let Ok(f) = file {
                    let p = f.into_path().to_str().unwrap().to_string();
                    if let Ok(disk) = Disk::new(&p, true /* quick */, None) {
                        result.push(disk.disk_info().clone());
                    } else {
                        ui_log(&format!("Error getting disk_info: {p}"));
                    }
                } else {
                    ui_log(&format!("Error in file {:?}", file));
                }
            }
        };

        result.sort_by(|a, b| a.name().to_lowercase().partial_cmp(&b.name().to_lowercase()).unwrap());
        result.dedup_by(|a, b| a.name() == b.name());
        result
    }
}


