use crate::constants::DEFAULT_EMULATOR_SPEED_HZ;

#[derive(Clone, Debug)]
pub struct WatchedFileMsg {
    pub path: String,
    pub address: u16,
    /// If Some, jump to that address as soon as the file is loaded
    pub starting_address: Option<u16>,
}

impl WatchedFileMsg {
    pub fn copy(&self) -> Self {
        Self { path: self.path.clone(), address: self.address, starting_address: self.starting_address }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub debug_asm: bool,
    pub csv: bool,
    pub trace_to_file: bool,
    pub trace_file_csv: String,
    pub trace_file_asm: String,
    /// Trace everything between these two addresses
    pub trace_range: Option<(u16, u16)>,
    pub trace_count: Option<u64>,

    /// Start tracing when the PC hits that address
    pub trace_pc_start: Option<u16>,
    /// ... and stop tracing when it reaches that one
    pub trace_pc_stop: Option<u16>,

    /// Start tracing after that many cycles
    pub trace_cycles_start: u128,

    pub watched_files: Vec<WatchedFileMsg>,
    pub is_65c02: bool,
    pub emulator_speed_hz: u64,
    pub asynchronous_logging: bool,
}

impl Config {
    pub fn copy(&self) -> Self {
        Self {
            debug_asm: self.debug_asm,
            csv: self.csv,
            trace_to_file: self.trace_to_file,
            trace_file_csv: self.trace_file_csv.clone(),
            trace_file_asm: self.trace_file_asm.clone(),
            trace_range: self.trace_range,
            trace_count: self.trace_count,
            trace_pc_start: self.trace_pc_start,
            trace_pc_stop: self.trace_pc_stop,
            trace_cycles_start: self.trace_cycles_start,
            watched_files: {
                let mut result: Vec<WatchedFileMsg> = Vec::new();
                for wf in &self.watched_files {
                    result.push(wf.copy());
                }
                result
            },
            is_65c02: self.is_65c02,
            emulator_speed_hz: self.emulator_speed_hz,
            asynchronous_logging: true,
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            debug_asm: false,
            // debug_asm: true,
            trace_to_file: false,
            // trace_to_file: true,
            csv: false,
            trace_file_csv: "c:\\t\\trace.csv".to_string(),
            trace_file_asm: "c:\\t\\trace.txt".to_string(),
            trace_range: None, // Some((0x801, 0x900)),
            trace_count: None, // Some(1_000_000),
            trace_pc_start: None, // Some(0x9777),
            trace_pc_stop: None,
            trace_cycles_start: 0, // 24_000_291,
            watched_files: Vec::new(),
            is_65c02: false,
            emulator_speed_hz: DEFAULT_EMULATOR_SPEED_HZ,
            asynchronous_logging: false,
        }
    }
}
