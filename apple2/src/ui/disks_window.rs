use std::path::{Path};
use eframe::egui::{Align, Color32, Label, Layout, RichText, ScrollArea, Ui, Vec2};
use ignore::{DirEntry, Walk};
use rfd::FileDialog;
use crate::constants::{BUGGY_DISKS, DISKS_SUFFIXES};
use crate::disk::disk_controller::{DiskController};
use crate::messages::ToCpu::LoadDisk;
use crate::ui::ui::{MyEguiApp, ui_log};

#[derive(Clone)]
pub(crate) struct DisplayedDisk {
    file_name: String,
    path: String,
}

impl DisplayedDisk {
    fn new(d: DirEntry) -> DisplayedDisk {
        DisplayedDisk {
            file_name: d.file_name().to_str().unwrap().to_string(),
            path: d.path().to_str().unwrap().to_string(),
        }
    }
}

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
            if self.disks_filter.is_empty() || d.file_name.to_lowercase().contains(&self.disks_filter) {
                Some(d.clone())
            } else {
                None
            }
        }).collect::<Vec<DisplayedDisk>>();
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
                        for displayed_disk in &self.disks_displayed_disks {
                            ui.horizontal(|ui| {
                                //
                                // Drive 1 / Drive 2
                                //
                                let mut drive_label = |index: usize, label: &str| {
                                    let mut drive_label = RichText::new(label);
                                    if let Some(di) = &self.disk_infos[index] {
                                        if *di.path == *displayed_disk.path {
                                            drive_label = RichText::new(label)
                                                .background_color(Color32::DARK_BLUE);
                                        }
                                    }

                                    if ui.button(drive_label).clicked() {
                                        let disk = DiskController::
                                            load_disk_new(&displayed_disk.path, None);
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
                                let color = if Self::is_buggy(&displayed_disk.file_name) {
                                    Color32::RED
                                } else if displayed_disk.file_name.to_lowercase().ends_with("woz") {
                                    Color32::YELLOW
                                } else {
                                    Color32::LIGHT_BLUE
                                };
                                ui.scope(|ui| {
                                    ui.set_max_width(150.0);
                                    let max = std::cmp::min(60, displayed_disk.file_name.len());
                                    let n = &displayed_disk.file_name[0..max];
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

    pub fn read_disks_directories(directories: &[String]) -> Vec<DisplayedDisk> {
        let mut result: Vec<DisplayedDisk> = Vec::new();
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
                    result.push(DisplayedDisk::new(f));
                } else {
                    ui_log(&format!("Error in file {:?}", file));
                }
            }
        };

        result.sort_by(|a, b| a.file_name.to_lowercase().partial_cmp(
            &b.file_name.to_lowercase()).unwrap());
        result.dedup_by(|a, b| a.file_name == b.file_name);
        result
    }
}


