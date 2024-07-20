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
pub(crate) const DEFAULT_SPEED_HZ: u64 = 1_300_000;

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
pub(crate) static CYCLES: RwLock<u128> = RwLock::new(0);
pub(crate) fn pc() -> String {
    format!("{:04X}", *PC.read().unwrap())
}

pub(crate) const DEFAULT_DISK_INDICES: [Option<usize>; 2] =
    [Some(33,  /* 39 pics 40 KQ, 17 DOS, 41 bad */), Some(39)];

// Disks
lazy_static! {
    pub static ref ALL_DISKS: Vec<DiskInfo> = vec![
        DiskInfo::new("DOS 3.3", "d:\\Apple disks\\Apple DOS 3.3.dsk"), // 0
        DiskInfo::new("Dos 3.3 August 1980", "d:\\Apple disks\\Apple DOS 3.3 August 1980.dsk"), // 1
        DiskInfo::new("NTSC", "d:\\Apple disks\\ntsc.dsk"), // 2
        DiskInfo::new("master", "d:\\Apple disks\\master.dsk"), // 3
        DiskInfo::new("Sherwood Forest", "d:\\Apple disks\\Sherwood_Forest.dsk"),  // 4
        DiskInfo::new("A2Audit", "C:\\Users\\Ced\\kotlin\\sixty\\src\\test\\resources\\audit.dsk"), // 5
        DiskInfo::new("ProDOS 2.4.1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\ProDOS_2_4_1.dsk"), // 6
        DiskInfo::new("Cedric", "d:\\Apple disks\\cedric.dsk"), // 7
        DiskInfo::new("Transylvania *", "d:\\Apple disks\\TRANS1.DSK"), // 8
        DiskInfo::new("Masquerade - 1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Masquerade-1.dsk"), // 9
        DiskInfo::new("Masquerade - 2", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Masquerade-2.dsk"), // 10
        DiskInfo::new("Ultima 4 - 1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Ultima4.dsk"), // 11
        DiskInfo::new("Blade of Blackpoole - 1", "d:\\Apple disks\\The Blade of Blackpoole side A.woz"), // 12
        DiskInfo::new("Blade of Blackpoole - 2", "d:\\Apple disks\\The Blade of Blackpoole side B.woz"), // 13
        DiskInfo::new("The Coveted Mirror - 1", "d:\\Apple disks\\COVETED1.DSK"), // 14
        DiskInfo::new("The Coveted Mirror - 2", "d:\\Apple disks\\COVETED2.DSK"), // 15
        DiskInfo::new("Sherwood Forest", "d:\\Apple disks\\Sherwood Forest.woz"), // 16
        DiskInfo::new("Apple DOS 3.3", "d:\\Apple disks\\DOS 3.3.woz"), // 17
        DiskInfo::new("Blazing Paddles", "d:\\Apple disks\\Blazing Paddles (Baudville).woz"), // 18
        DiskInfo::new("Bouncing Kamungas", "d:\\Apple disks\\Bouncing Kamungas.woz"), // 19
        DiskInfo::new("Commando - 1", "d:\\Apple disks\\Commando - DIsk 1, Side A.woz"), // 20
        DiskInfo::new("Night Mission Pinball", "d:\\Apple disks\\Night Mission Pinball.woz"), // 21
        DiskInfo::new("Rescue Raiders", "d:\\Apple disks\\Rescue Raiders - Disk 1, Side B.woz"), // 22
        DiskInfo::new("Karateka", "d:\\Apple disks\\Karateka.dsk"), // 23
        DiskInfo::new("Dark Lord - 1", "d:\\Apple disks\\Dark Lord side A.woz"), // 24
        DiskInfo::new("Dark Lord - 2", "d:\\Apple disks\\Dark Lord side B.woz"), // 25
        DiskInfo::new("Sammy Lightfoot", "d:\\Apple disks\\Sammy Lightfoot - Disk 1, Side A.woz"), // 26
        DiskInfo::new("Stargate - 1 *", "d:\\Apple disks\\Stargate - Disk 1, Side A.woz"), // 27
        DiskInfo::new("Stellar 7", "d:\\Apple disks\\Stellar 7.woz"), // 28
        DiskInfo::new("Aztec", "d:\\Apple disks\\Aztec (4am crack).dsk"), // 29
        DiskInfo::new("Aztec", "d:\\Apple disks\\Aztec.woz"), // 30
        DiskInfo::new("Conan - 1", "d:\\Apple disks\\Conan side A.woz"), // 31
        DiskInfo::new("Conan - 2", "d:\\Apple disks\\Conan side B.woz"), // 32
        DiskInfo::new("Adventureland - 1", "d:\\Apple disks\\Adventureland - 1.woz"), // 33
        DiskInfo::new("Adventureland - 2", "d:\\Apple disks\\Adventureland - 2.woz"), // 34
        DiskInfo::new("Arctic Fox", "d:\\Apple disks\\Arcticfox.woz"), // 35
        DiskInfo::new("Frogger", "d:\\Apple disks\\Frogger.woz"), // 36
        DiskInfo::new("Demo by Five Touch", "d:\\Apple disks\\patched.woz"), // 37
        DiskInfo::new("Wizardry 1", "d:\\Apple disks\\Wizardry 1 - 1.woz"), // 38
        DiskInfo::new("a2fc", "d:\\Apple disks\\A2BestPix_Top1.DSK"), // 39
        DiskInfo::new("King's Quest 1 - 1", "d:\\Apple disks\\King's Quest - A.woz"), // 40
        DiskInfo::new("test", "c:\\Users\\Ced\\rust\\sixty.rs\\bad.woz"), // 41
        DiskInfo::new("Star Trek - 1", "d:\\Apple disks\\Star Trek - First Contact - Disk 1, Side 1.woz"), // 42
        DiskInfo::n("d:\\Apple disks\\Airheart.woz"), // 43
        DiskInfo::n("d:\\Apple disks\\Apple Galaxian.woz"), // 44
    ];

    /// Disks that are not booting or not working properly
    pub static ref BUGGY_DISKS: Vec<String> = vec![
        "Bruce Lee".to_string(), // Requires joystick
        "Crystal Quest".to_string(),
        "Frogger".to_string(),
        "Maniac Mansion".to_string(), // Infinite loop at $204
        "Prince of Persia".to_string(),
        "Stargate".to_string(),
        "DOS 3.2".to_string(),
        "Zork r5".to_string(),
        "Akalabeth".to_string(),
        "Ankh".to_string(),
        "Batman".to_string(),
        "Algernon".to_string(),  // broken if q6 and q7 are tested in lss::step()
        "Micro invaders".to_string(),
        "F-15".to_string(), // F-15 Strike Eagle
        "Drol".to_string(),
        "Bug Attack".to_string(),
        "Seafox".to_string(), // https://github.com/AppleWin/AppleWin/issues/668
        "Micro Invaders".to_string(),
        // Newsroom is up there with Wizardry and Sun Dog for the most non-working disks in my experience. I keep trying to image new copies but only half the time does the program disk work.
        // It requires a custom bit timing
        // On the woz
        // For the entire disk
        "Newsroom".to_string(),
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

