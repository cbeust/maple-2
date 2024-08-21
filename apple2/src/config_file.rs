use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use cpu::constants::DEFAULT_EMULATOR_SPEED_HZ;

use crate::constants::{DEFAULT_DISKS_DIRECTORIES, DEFAULT_MAGNIFICATION, DEFAULT_SPEED_HZ};
use crate::roms::RomType;
use crate::ui_log;

/// Name of the config file, which is saved under whatever dirs::config_dir() returns
pub(crate) const CONFIG_DIR: &str = "maple2";
pub(crate) const CONFIG_FILE: &str = "config.json";

fn as_true() -> bool { true }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Breakpoint {
    pub(crate) address: u16,
    #[serde(default = "as_true")]
    pub(crate) enabled: bool,
}

/// Saved in the user settings file `CONFIG_FILE` in the config directory.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    emulator_speed_hz: u64,
    disk_directories: Vec<String>,
    drive_1: Option<String>,
    drive_2: Option<String>,
    hard_drive_1: Option<String>,
    hard_drive_2: Option<String>,
    // The tab to start in (ordinal position of the enum [MainTab]
    pub(crate) tab: usize,
    pub(crate) magnification: Option<u16>,

    #[serde(default)]
    breakpoints: Vec<Breakpoint>,

    #[serde(skip)]
    pub(crate) breakpoints_hash: HashSet<u16>,
    rom_type: Option<RomType>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            emulator_speed_hz: DEFAULT_SPEED_HZ,
            magnification: Some(DEFAULT_MAGNIFICATION),
            disk_directories: Vec::new(),
            drive_1: None,
            drive_2: None,
            hard_drive_1: None,
            hard_drive_2: None,
            tab: 0,
            breakpoints: Vec::new(),
            breakpoints_hash: HashSet::new(),
            rom_type: Some(RomType::Apple2Enhanced),
        }
    }
}

impl ConfigFile {
    pub fn magnification(&self) -> u16 { self.magnification.unwrap_or_else(|| DEFAULT_MAGNIFICATION)}

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

        result.recalculate();
        result
    }

    pub fn rom_type(&self) -> RomType {
        match &self.rom_type {
            None => { RomType::Apple2Enhanced }
            Some(rt) => { rt.clone() }
        }
    }

    pub fn hard_drive_1(&self) -> Option<String> {
        self.hard_drive_1.clone()
    }

    pub fn hard_drive_2(&self) -> Option<String> {
        self.hard_drive_2.clone()
    }

    pub fn breakpoints(&self) -> &Vec<Breakpoint> {
        &self.breakpoints
    }

    pub(crate) fn add_breakpoint(&mut self, bp: String) {
        match u16::from_str_radix(&bp, 16) {
            Ok(address) => {
                let mut vec = self.breakpoints().clone();
                vec.push(Breakpoint {
                    address,
                    enabled: true,
                });
                self.breakpoints.clear();
                self.breakpoints.clone_from(&vec);
                self.recalculate();
                self.save();
            }
            Err(_) => { ui_log(&format!("Couldn't add breakpoint {bp}")); }
        }
    }

    fn recalculate(&mut self) {
        self.breakpoints_hash.clear();
        for bp in &self.breakpoints {
            self.breakpoints_hash.insert(bp.address);
        }
    }

    pub(crate) fn disk_directories(&self) -> &Vec<String> { &self.disk_directories }
    pub fn emulator_speed_hz(&self) -> u64 { self.emulator_speed_hz }
    pub(crate) fn drive_1(&self) -> Option<String> { self.drive_1.clone() }
    pub(crate) fn drive_2(&self) -> Option<String> { self.drive_2.clone() }

    pub fn set_drive(&mut self, is_hard_drive: bool, drive_number: usize, path: Option<String>) {
        if let Some(p) = &path {
            if p.ends_with("hdv") && ! is_hard_drive {
                println!("BUG HARD DRIVE");
            }
        }
        match (is_hard_drive, drive_number) {
            (true, 0) => { self.hard_drive_1 = path }
            (true, 1) => { self.hard_drive_2 = path }
            (false, 0) => { self.drive_1 = path }
            (false, 1) => { self.drive_2 = path }
            _ => { panic!("Should never happen"); }
        }
        self.save();
    }

    pub fn delete_breakpoint(&mut self, address: u16) {
        if let Some(index) = self.breakpoints.iter().position(|bp| bp.address == address) {
            self.breakpoints.remove(index);
        }
        self.recalculate();
        self.save();
    }

    pub fn set_tab(&mut self, tab_index: usize) {
        self.tab = tab_index;
        self.save();
    }

    pub(crate) fn set_disk_directories(&mut self, directories: Vec<String>) {
        self.disk_directories = directories.clone();
        self.save();
    }

    fn save(&self) {
        if let Some(config_file_path) = Self::config_file_path() {
            if let Ok(serialized) = serde_json::to_string_pretty(&self) {
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
                let user_config = ConfigFile {
                    emulator_speed_hz: DEFAULT_EMULATOR_SPEED_HZ,
                    disk_directories: existing,
                    drive_1: None, drive_2: None,
                    hard_drive_1: None,
                    hard_drive_2: None,
                    tab: 0,
                    magnification: Some(DEFAULT_MAGNIFICATION),
                    breakpoints: Vec::new(),
                    breakpoints_hash: HashSet::new(),
                    rom_type: Some(RomType::Apple2Enhanced),
                };
                user_config.save();
            }
        } else {
            ui_log("Config directory doesn't seem to exist, not saving settings");
        }
    }
}