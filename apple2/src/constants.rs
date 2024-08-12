use std::sync::{OnceLock, RwLock};
use std::time::Instant;
use crossbeam::channel::Sender;
use lazy_static::lazy_static;
use cpu::config::WatchedFileMsg;
use crate::disk::disk_info::DiskInfo;
use crate::messages::ToUi;

/// We run 100_000 cycles every 100 ms, so 10 times a second => 1 Mhz
// pub(crate) const _EMULATOR_PERIOD_CYCLES: u64 = 100_000;
// pub(crate) const _EMULATOR_PERIOD_MS: u64 =
//     1000 * _EMULATOR_PERIOD_CYCLES / DEFAULT_EMULATOR_SPEED_HZ;

/// How often we send a CpuDump message to the UI
pub(crate) const CPU_REFRESH_MS: u128 = 40;

///
/// Screen sizes
///

pub(crate) const TEXT_WIDTH: u8 = 40;
pub(crate) const TEXT_HEIGHT: u8 = 24;

pub(crate) const HIRES_WIDTH: u16 = 280;
pub(crate) const HIRES_HEIGHT_MIXED: u16 = 160;
pub(crate) const HIRES_HEIGHT: u16 = 192;

pub const _FRAMES_PER_SECOND: usize = 50;
pub const _CYCLES_PER_LINE: usize = 65;
pub const _CYCLES_PER_FRAME: usize = 20280;


///
/// Font sizes
///
pub(crate) const FONT_WIDTH: u8 = 8;
pub(crate) const FONT_HEIGHT: u8 = 7;

///
/// Offset addresses for text mode
///
pub(crate) const TEXT_MODE_ADDRESSES: [u16; TEXT_HEIGHT as usize] = [
    0x400, 0x480, 0x500, 0x580, 0x600, 0x680, 0x700, 0x780,
    0x428, 0x4a8, 0x528, 0x5a8, 0x628, 0x6a8, 0x728, 0x7a8,
    0x450, 0x4d0, 0x550, 0x5d0, 0x650, 0x6d0, 0x750, 0x7d0,
];

/// Frequency of the text FLASH mode in Hz (e.g. 4 = 4 times a second). Even number.
pub const FLASH_FREQUENCY_HZ: u8 = 4;

/// Magnification of the screen
pub(crate) const DEFAULT_MAGNIFICATION: u16 = 4;
pub(crate) const DEFAULT_SPEED_HZ: u64 = 1_740_000;

/// Disk controller: read bytes, bits, or use the LSS
#[derive(PartialEq)]
pub(crate) enum NibbleStrategy { Bytes, Bits, Lss }
pub(crate) const NIBBLE_STRATEGY: NibbleStrategy = NibbleStrategy::Lss;
pub(crate) const HOLD: u16 = 81;

/// How many cycles to wait between the time when the motor is turned off
/// and when it actually turns off
pub(crate) const SPINNING_DOWN_CYCLES: u64 = 1_200_000;

/// Globals mostly used for debugging and getting access to the current PC and the start time from anywhere
/// Wish I had some decent Dependency Injection instead of having to declare globals :-(
pub(crate) static START: OnceLock<Instant> = OnceLock::new();
pub(crate) static SENDER_TO_UI: OnceLock<Sender<ToUi>> = OnceLock::new();
pub(crate) static PC: RwLock<u16> = RwLock::new(0);
pub(crate) static CYCLES: RwLock<u64> = RwLock::new(0);
pub(crate) fn pc() -> String {
    format!("{:04X}", *PC.read().unwrap())
}

lazy_static! {
    /// Disks that are not booting or not working properly
    pub static ref BUGGY_DISKS: Vec<String> = vec![
        "Stargate".to_string(), // I/O error
        "DOS 3.2".to_string(),
        "Akalabeth".to_string(),  // 3.2?
        "Batman".to_string(),
        "Algernon".to_string(),  // broken if q6 and q7 are tested in lss::step()
        "Micro invaders".to_string(), // 13 sectors (boot format = 2)
        "Bug Attack".to_string(),
        "Seafox".to_string(), // https://github.com/AppleWin/AppleWin/issues/668
        "Micro Invaders".to_string(),
        // Newsroom is up there with Wizardry and Sun Dog for the most non-working disks in my
        // experience. I keep trying to image new copies but only half the time does the
        // program disk work.
        // It requires a custom bit timing on the woz for the entire disk (need FLUX)
        "Newsroom".to_string(),
        "Pest Patrol".to_string(),
    ];

    pub static ref DEFAULT_DISKS_DIRECTORIES: Vec<String> = vec![];

    pub static ref DISKS_SUFFIXES: [String; 3] = [
        "woz".to_string(), "dsk".to_string(), "hdv".to_string()
    ];

    pub static ref WATCHED_FILES: Vec<WatchedFileMsg> = vec![
        // WatchedFile {
        //     path: "d:\\t\\pic.hgr".to_string(),
        //     address: 0x2000,
        //     starting_address: None,
        // },
        // WatchedFile {
        //     path: "d:\\t\\pic.hgr.aux".to_string(),
        //     address: 0x2000,
        //     starting_address: None,
        // }
    ];

}

