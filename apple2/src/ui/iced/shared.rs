use std::collections::VecDeque;
use std::ops::DerefMut;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use once_cell::sync::Lazy;

use cpu::cpu::RunStatus;

use crate::disk::disk_info::DiskInfo;
use crate::joystick::Joystick;
use crate::messages::CpuDumpMsg;

#[derive(Default)]
struct Drive {
    disk_info: Option<DiskInfo>,
    sector: u8,
    phase_160: u8,
}

#[derive(Default)]
struct HardDrive {
    disk_info: Option<DiskInfo>,
    block_number: u16,
}

static DRIVES: [RwLock<Lazy<Drive>>; 2] = [
    RwLock::new(Lazy::new(|| Drive::default())),
    RwLock::new(Lazy::new(|| Drive::default()))
];

static HARD_DRIVES: [RwLock<Lazy<HardDrive>>; 2] = [
    RwLock::new(Lazy::new(|| HardDrive::default())),
    RwLock::new(Lazy::new(|| HardDrive::default()))
];

static BREAKPOINT_WAS_HIT: RwLock<bool> = RwLock::new(false);

struct CpuHolder {
    cpu: CpuDumpMsg,
}

static CPU: RwLock<Lazy<CpuHolder>> = RwLock::new(Lazy::new(|| CpuHolder { cpu: CpuDumpMsg::default() }));

#[derive(Copy, Clone)]
pub struct SpeakerEvent {
    pub cycle: u64,
    pub timestamp: u64,
}

static SPEAKER_EVENTS: RwLock<Vec<SpeakerEvent>> = RwLock::new(Vec::new());
static SOUND_SAMPLES: RwLock<VecDeque<f32>> = RwLock::new(VecDeque::new());
static LAST_SAMPLE_PLAYED: RwLock<Option<(Instant, f32)>> = RwLock::new(None);
static SHOW_DRIVES: RwLock<bool> = RwLock::new(true);

#[derive(Default)]
struct SharedJoystick {
    reset: [u64; 4],
    values: [u8; 4],
    buttons: [bool; 4],
}

static JOYSTICK: RwLock<Lazy<Joystick>> = RwLock::new(Lazy::new(|| Joystick::default()));
static SHARED_JOYSTICK: RwLock<Lazy<SharedJoystick>> = RwLock::new(Lazy::new(|| SharedJoystick::default()));

pub struct Shared;

impl Shared {
    pub fn get_cpu() -> CpuDumpMsg { CPU.read().unwrap().cpu.clone() }
    pub fn set_cpu(cpu: CpuDumpMsg) { CPU.write().unwrap().cpu = cpu; }
    pub fn set_run_status(run_status: RunStatus) { CPU.write().unwrap().cpu.run_status = run_status; }

    pub fn get_breakpoint_was_hit() -> bool {
        *BREAKPOINT_WAS_HIT.read().unwrap()
    }

    pub fn set_breakpoint_was_hit(v: bool) {
        *BREAKPOINT_WAS_HIT.write().unwrap() = v;
    }

    pub fn get_phase_160(drive_index: usize) -> u8 {
        DRIVES[drive_index].read().unwrap().phase_160
    }

    pub fn set_phase_160(drive_index: usize, phase_160: u8) {
        DRIVES[drive_index].write().unwrap().phase_160 = phase_160;
    }

    pub fn get_track(drive_index: usize) -> u8 {
        Self::get_phase_160(drive_index) / 4
    }

    pub fn get_block_number(drive_index: usize) -> u16 {
        HARD_DRIVES[drive_index].read().unwrap().block_number
    }

    pub fn set_block_number(drive_index: usize, n: u16) {
        HARD_DRIVES[drive_index].write().unwrap().block_number = n;
    }

    pub fn get_sector(drive_index: usize) -> u8 {
        DRIVES[drive_index].read().unwrap().sector
    }

    pub fn set_sector(drive_index: usize, sector: u8) {
        DRIVES[drive_index].write().unwrap().sector = sector;
    }

    pub fn get_drive(drive_index: usize) -> Option<DiskInfo> {
        if let Ok(r) = DRIVES[drive_index].try_read() {
            r.disk_info.clone()
        } else {
            None
        }
    }

    pub fn set_drive(drive_index: usize, disk_info: Option<DiskInfo>) {
        DRIVES[drive_index].write().unwrap().deref_mut().disk_info = disk_info;
    }


    pub fn get_hard_drive(drive_index: usize) -> Option<DiskInfo> {
        if let Ok(r) = HARD_DRIVES[drive_index].try_read() {
            r.disk_info.clone()
        } else {
            None
        }
    }

    pub fn get_hard_drive_name(drive_index: usize) -> String {
        Shared::get_hard_drive(drive_index).map_or("".to_string(), |di| di.name().to_string())
    }

    pub fn set_hard_drive(drive_index: usize, disk_info: Option<DiskInfo>) {
        HARD_DRIVES[drive_index].write().unwrap().deref_mut().disk_info = disk_info;
    }

    pub fn add_speaker_event(event: SpeakerEvent) { SPEAKER_EVENTS.write().unwrap().push(event); }
    pub fn get_speaker_events() -> Vec<SpeakerEvent> {
        let result = SPEAKER_EVENTS.read().unwrap().clone();
        SPEAKER_EVENTS.write().unwrap().clear();
        result
    }

    pub fn get_next_sound_sample_maybe() -> Option<f32> {
        SOUND_SAMPLES.write().unwrap().pop_front()
    }

    pub fn has_samples() -> bool {
        ! SOUND_SAMPLES.read().unwrap().is_empty()
    }

    pub fn add_sound_sample(s: f32) {
        SOUND_SAMPLES.write().unwrap().push_back(s);
    }

    pub fn set_last_sample_played(s: Option<(Instant, f32)>) {
        *LAST_SAMPLE_PLAYED.write().unwrap() = s;
    }

    pub fn get_last_sample_played() -> Option<(Instant, f32)>{
        *LAST_SAMPLE_PLAYED.read().unwrap()
    }

    pub fn get_show_drives() -> bool {
        *SHOW_DRIVES.read().unwrap()
    }

    pub fn set_show_drives(b: bool) {
        *SHOW_DRIVES.write().unwrap() = b;
    }

    pub fn reset_joystick(cycle: u64) {
        JOYSTICK.write().unwrap().reset_cycles(cycle);
    }

    pub fn get_controller_value(index: usize, cycle: u64) -> u8 {
        JOYSTICK.write().unwrap().get_value_for_paddle(index, cycle)
    }

    pub fn update_controller_raw_values(values: [u8; 4]) {
        SHARED_JOYSTICK.write().unwrap().values = values;
    }

    pub fn get_controller_raw_values() -> [u8; 4] {
        SHARED_JOYSTICK.read().unwrap().values.clone()
    }

    pub fn get_controller_button_value(index: usize) -> bool {
        SHARED_JOYSTICK.read().unwrap().buttons[index]
    }

    pub fn set_controller_button_value(index: usize, v: bool) {
        SHARED_JOYSTICK.write().unwrap().buttons[index] = v;
    }
}
