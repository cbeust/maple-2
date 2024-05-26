use std::fs::File;
use std::io::prelude::*;
use cpu::config::WatchedFileMsg;
use cpu::cpu::StatusFlags;
use cpu::disassembly::Disassemble;
use cpu::operand::Operand;
use crate::apple2_cpu::EmulatorConfigMsg;
use crate::disk::disk_info::DiskInfo;
use crate::disk::drive::DriveStatus;
use crate::ui::ui::DrawCommand;

#[derive(Default)]
pub struct CpuDumpMsg {
    pub memory: Vec<u8>,
    pub aux_memory: Vec<u8>,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub p: StatusFlags,
    pub s: u8,
}

#[derive(Default, Clone)]
pub struct TrackSectorMsg {
    pub track: f32,
    pub sector: u8,
}

pub enum ToUi {
    Config(EmulatorConfigMsg),
    CpuDump(CpuDumpMsg),
    EmulatorSpeed(f32),  // Speed in Mhz
    // Drive (0 or 1), track, sector
    TrackSector(usize, TrackSectorMsg),
    // Drive (0 or 1), disk info (file name and name)
    DiskInfo(usize, Option<DiskInfo>),
    // Different drive selected (0 or 1)
    DiskSelected(usize),
    // Tell the UI that we're ready for another key
    KeyboardStrobe,
    // First parameter: drive (0 or 1)
    DriveMotorStatus(usize, DriveStatus),
    // Drive (0 or 1), phase80
    PhaseUpdate(usize, u8),
    // Whenever the RGB mode is changed
    RgbModeUpdate(u8),
}

#[derive(Default)]
pub struct SetMemoryMsg {
    pub address: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum CpuStateMsg {
    Running, Paused,
}

#[derive(Default)]
pub struct TraceStatusMsg {
    pub debug_asm: Option<bool>,
    pub trace_file: Option<bool>,
    pub trace_file_csv: Option<String>,
    pub trace_file_asm: Option<String>,
    pub csv: Option<bool>,
}

pub struct GenerateDisassemblyMsg {
    pub from: u16,
    pub to: u16,
    pub filename: String,
}

impl GenerateDisassemblyMsg {
    pub fn generate(&self, memory: &[u8], operands: &[Operand]) {
        let lines = Disassemble::disassemble_range(memory, operands, self.from as usize, self.to as usize);
        match File::create(&self.filename) {
            Ok(mut file) => {
                for l in lines {
                    file.write_all(l.to_asm().as_bytes()).unwrap();
                    file.write_all(b"\n").unwrap();
                }
                println!("Generated disassembly in {}", self.filename);
            }
            Err(error) => {
                println!("Couldn't open file {}: {}", self.filename, error);
            }
        }

    }
}

pub enum ToCpu {
    SetMemory(SetMemoryMsg),
    GetMemory(u16),
    FileModified(WatchedFileMsg),
    SwapDisks,
    Reboot,
    /// Save $2000 to a file
    SaveGraphics,
    /// Drive number (0 or 1), path
    LoadDisk(usize, DiskInfo),
    Debug,
    /// true: run, false: pause
    CpuState(CpuStateMsg),
    TraceStatus(TraceStatusMsg),
    GenerateDisassembly(GenerateDisassemblyMsg),
}

pub enum ToMiniFb {
    Buffer(Vec<DrawCommand>),
}
