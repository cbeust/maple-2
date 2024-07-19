use std::ops::DerefMut;
use std::string::ToString;
use std::sync::{RwLock, RwLockReadGuard};
use once_cell::sync::Lazy;
use cpu::cpu::RunStatus;
use crate::disk::disk_info::DiskInfo;
use crate::messages::CpuDumpMsg;

#[derive(Default)]
struct Drive {
    disk_info: Option<DiskInfo>,
    track: u8,
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

pub struct Shared;

impl Shared {
    pub fn cpu() -> CpuDumpMsg { CPU.read().unwrap().cpu.clone() }
    pub fn set_cpu(cpu: CpuDumpMsg) { CPU.write().unwrap().cpu = cpu; }
    pub fn set_run_status(run_status: RunStatus) { CPU.write().unwrap().cpu.run_status = run_status; }

    pub fn breakpoint_was_hit() -> bool {
        *BREAKPOINT_WAS_HIT.read().unwrap()
    }

    pub fn set_breakpoint_was_hit(v: bool) {
        *BREAKPOINT_WAS_HIT.write().unwrap() = v;
    }

    pub fn phase_160(drive_index: usize) -> u8 {
        DRIVES[drive_index].read().unwrap().phase_160
    }

    pub fn set_phase_160(drive_index: usize, phase_160: u8) {
        DRIVES[drive_index].write().unwrap().phase_160 = phase_160;
    }

    pub fn track(drive_index: usize) -> u8 {
        DRIVES[drive_index].read().unwrap().track
    }

    pub fn set_track(drive_index: usize, track: u8) {
        DRIVES[drive_index].write().unwrap().track = track;
    }

    pub fn block_number(drive_index: usize) -> u16 {
        HARD_DRIVES[drive_index].read().unwrap().block_number
    }

    pub fn set_block_number(drive_index: usize, n: u16) {
        HARD_DRIVES[drive_index].write().unwrap().block_number = n;
    }

    pub fn sector(drive_index: usize) -> u8 {
        DRIVES[drive_index].read().unwrap().sector
    }

    pub fn set_sector(drive_index: usize, sector: u8) {
        DRIVES[drive_index].write().unwrap().sector = sector;
    }

    pub fn drive(drive_index: usize) -> Option<DiskInfo> {
        if let Ok(r) = DRIVES[drive_index].try_read() {
            r.disk_info.clone()
        } else {
            None
        }
    }

    pub fn set_drive(drive_index: usize, disk_info: Option<DiskInfo>) {
        DRIVES[drive_index].write().unwrap().deref_mut().disk_info = disk_info;
    }


    pub fn hard_drive(drive_index: usize) -> Option<DiskInfo> {
        if let Ok(r) = HARD_DRIVES[drive_index].try_read() {
            r.disk_info.clone()
        } else {
            None
        }
    }

    pub fn hard_drive_name(drive_index: usize) -> String {
        Shared::hard_drive(drive_index).map_or("".to_string(), |di| di.name().to_string())
    }

    pub fn set_hard_drive(drive_index: usize, disk_info: Option<DiskInfo>) {
        HARD_DRIVES[drive_index].write().unwrap().deref_mut().disk_info = disk_info;
    }

}