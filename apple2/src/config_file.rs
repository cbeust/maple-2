use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use cpu::constants::DEFAULT_EMULATOR_SPEED_HZ;
use crate::constants::{ALL_DISKS, DEFAULT_DISK_INDICES, DEFAULT_DISKS_DIRECTORIES};
use crate::ui::ui::{MainTab, ui_log};

/// Name of the config file, which is saved under whatever dirs::config_dir() returns
pub(crate) const CONFIG_DIR: &str = "maple2";
pub(crate) const CONFIG_FILE: &str = "config.json";

/// Saved in the user settings file `CONFIG_FILE` in the config directory.
#[derive(Clone, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    emulator_speed_hz: u64,
    disk_directories: Vec<String>,
    drive_1: Option<String>,
    drive_2: Option<String>,
    // The tab to start in (ordinal position of the enum [MainTab]
    pub(crate) tab: usize,
}

impl ConfigFile {
    /// If a ConfigFile already exists, read it, if not create it with defaults, then return it
    pub fn new() -> ConfigFile {
        let mut result = ConfigFile::default();
        Self::maybe_create_config_file();
        if let Some(config_file) = Self::config_file_path() {
            let config_file_path = config_file.to_str().unwrap();
            if let Ok(string) = fs::read_to_string(config_file_path) {
                match serde_json::from_str::<ConfigFile>(&string) {
                    Ok(config_file) => {
                        ui_log(&format!("Found config file at {}", config_file_path));
                        result = config_file;
                    }
                    Err(err) => {
                        ui_log(&format!("Couldn't parse settings: {err}"));
                        result = ConfigFile::default();
                    }
                }
            }
        };

        result
    }

    pub(crate) fn disk_directories(&self) -> &Vec<String> { &self.disk_directories }
    pub fn emulator_speed_hz(&self) -> u64 { self.emulator_speed_hz }
    pub(crate) fn drive_1(&self) -> Option<String> { self.drive_1.clone() }
    pub(crate) fn drive_2(&self) -> Option<String> { self.drive_2.clone() }

    pub fn set_drive(&mut self, drive_number: usize, path: Option<String>) {
        if drive_number == 0 {
            self.drive_1 = path;
        } else {
            self.drive_2 = path;
        }
        Self::save_config(self);
    }

    pub fn set_tab(&mut self, tab: MainTab) {
        self.tab = tab.to_index();
        Self::save_config(self);
    }

    pub(crate) fn set_disk_directories(&mut self, directories: Vec<String>) {
        self.disk_directories = directories.clone();
        Self::save_config(self);
    }

    fn save_config(config_file: &ConfigFile) {
        if let Some(config_file_path) = Self::config_file_path() {
            if let Ok(serialized) = serde_json::to_string_pretty(&config_file) {
                let cf = config_file_path.to_str().unwrap();
                match File::create(config_file_path.clone()) {
                    Ok(mut file) => {
                        match file.write_all(serialized.as_ref()) {
                            Ok(_) => {
                                ui_log(&format!("Saved {}", cf));
                            }
                            Err(err) => {
                                ui_log(&format!("Couldn't create config file {}: {err}", cf));
                            }
                        }
                    }
                    Err(err) => {
                        ui_log(&format!("Couldn't create config file {}: {}", cf, err));
                    }
                }
            }
        } else {
            ui_log("Couldn't find config file, not saving");
        }
    }

    /// Return the fully qualified path + file name of the config file
    fn config_file_path() -> Option<PathBuf> {
        let mut result: Option<PathBuf> = None;

        let binding = dirs::config_dir().unwrap();
        if binding.exists() {
            let os_config_dir = binding.to_str().unwrap();
            let config_dir = Path::new(os_config_dir).join(CONFIG_DIR);
            if !config_dir.exists() {
                let _ = fs::create_dir(config_dir.clone());
            }
            result = Some(config_dir.join(CONFIG_FILE));
        }

        result
    }

    fn maybe_create_config_file() {
        if let Some(config_file) = Self::config_file_path() {
            if !config_file.exists() {
                let existing = DEFAULT_DISKS_DIRECTORIES.iter()
                    .filter(|s| Path::new(s).exists()).cloned()
                    .collect::<Vec<String>>();
                let drive_1 = DEFAULT_DISK_INDICES[0].map(|index| &ALL_DISKS[index])
                    .filter(|p| Path::new(&p.path).exists())
                    .map(|di| di.path.clone());
                let drive_2 = DEFAULT_DISK_INDICES[1].map(|index|& ALL_DISKS[index])
                    .filter(|p| Path::new(&p.path).exists())
                    .map(|di| di.path.clone());
                let user_config = ConfigFile {
                    emulator_speed_hz: DEFAULT_EMULATOR_SPEED_HZ,
                    disk_directories: existing,
                    drive_1, drive_2,
                    tab: 0,
                };
                Self::save_config(&user_config);
            }
        } else {
            ui_log("Config directory doesn't seem to exist, not saving settings");
        }
    }
}