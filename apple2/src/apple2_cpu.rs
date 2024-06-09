use std::fs;
use std::fs::File;
use std::io::Write;
use std::ops::{Add};
use crossbeam::channel::{Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log4rs::Handle;
use cpu::config::Config;
use cpu::cpu::{Cpu, RunStatus};
use cpu::memory::Memory;
use crate::memory::Apple2Memory;
use crate::messages::{CpuDumpMsg, CpuStateMsg, ToCpu, ToUi};
use crate::messages::ToUi::EmulatorSpeed;
use crate::misc::increase_cycles;
use crate::rolling_times::RollingTimes;
use crate::{configure_log, send_message};
use crate::config_file::ConfigFile;
use crate::constants::{PC, START};
use crate::ui::ui::ui_log;

#[derive(Clone, Default)]
pub struct EmulatorConfigMsg {
    pub config: Config,
    pub config_file: ConfigFile,
}

impl EmulatorConfigMsg {
    pub fn new(config: Config) -> Self {
        Self {
            config, ..Default::default()
        }
    }
}

pub struct AppleCpu {
    pub cpu: Cpu<Apple2Memory>,
    sender: Option<Sender<ToUi>>,
    receiver: Option<Receiver<ToCpu>>,
    last_memory_sent: Instant,
    last_cpu_run: Instant,
    /// Number of cycles run for this run (gets reset periodically)
    this_run_cycles: u64,
    rolling_times: RollingTimes,
    last_speed_sent: Instant,
    /// Cycle count to wait until processing next instruction
    wait: u8,
    config: EmulatorConfigMsg,
    cycles: u128,
    start: Instant,
    started: bool,
    handle: Option<Handle>,
}

impl AppleCpu {
    pub fn new(cpu: Cpu<Apple2Memory>, config: EmulatorConfigMsg, sender: Option<Sender<ToUi>>,
            receiver: Option<Receiver<ToCpu>>, handle: Option<Handle>) -> Self {
        Self { cpu, sender, receiver,
            last_memory_sent: Instant::now(),
            last_cpu_run: Instant::now(),
            this_run_cycles: 0,
            rolling_times: RollingTimes::new(),
            last_speed_sent: Instant::now(),
            wait: 0,
            cycles: 0,
            start: Instant::now(),
            started: false,
            config,
            handle,
        }
    }

    fn log(&self, s: &str) {
        // println!("[CPU {:>4}] {}", self.this_run_cycles, s);
    }

    pub fn step(&mut self) -> (bool, u64) {
        if ! self.started {
            self.start = Instant::now();
            self.started = true;
        }


        self.cpu.memory.disk_controller.step();
        let cycles = self.advance_cpu();
        *PC.write().unwrap() = self.cpu.pc;
        self.cpu.memory.disk_controller.step();

        self.cycles += cycles as u128;

        increase_cycles(1);

        (false, cycles)
    }

    fn advance_cpu(&mut self) -> u64 {
        let mut cycles = 0_u64;
        if self.wait == 0 {
            cycles = match self.cpu.step(&self.config.config) {
                RunStatus::Continue(c) => c as u64,
                RunStatus::Stop(_, _, _) => 0,
            };
            self.wait = (cycles - 1) as u8;
            self.this_run_cycles += cycles;
        } else {
            self.wait -= 1;
        }

        cycles
    }

    fn emulator_period_cycles(&self) -> u64 {
        100_000
    }

    fn emulator_period_ms(&self) -> u64 {
        1000 * self.emulator_period_cycles() / self.config.config.emulator_speed_hz
    }

    pub fn steps(&mut self, debug_asm: bool) -> (bool, u128) {
        let mut total_cycles: u128 = 0;
        let mut slice_cycles: u64 = 0;
        let stop = false;

        let slice_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        // It's 1one bit every 4 cpu cycles/8 lss cycles.  One nibble is between 32 and 40 cpu cycles
        // (64-80 lss cycles) usually, depending on the number of 0 sync bits.  As for the clearing
        // of the latch, it depends if the first two bits (post-optional-sync) are 10 or 11.
        // On 10 the latch is cleared 12 lss cycles after the first 1 (50% margin).
        // On 11 3 lss cycles after the second 1.
        while ! stop && slice_cycles < self.emulator_period_cycles() {
            let (st, cy) = self.step();
            // println!("Advancing {} cycles", cy);
            slice_cycles += cy;
            total_cycles += cy as u128;
        }
        self.rolling_times.add(slice_start,
                               SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
                               slice_cycles as u128);

        (false, total_cycles)
    }

    /// Return true if we're rebooting, false if we're exiting
    pub fn run(&mut self) -> bool {
        use ToCpu::*;
        let mut total_cycles: u128 = 0;
        let mut stop = false;
        let start = *START.get().unwrap();
        let mut next_cpu_run = (Instant::now() - Duration::new(10, 0)).duration_since(start).as_millis();
        let mut rebooting = false;
        let mut paused = false;

        while ! stop {
            if let Some(receiver) = &self.receiver {
                while ! stop && ! receiver.is_empty() {
                    if let Ok(message) = receiver.recv() {
                        match message {
                            SetMemory(sm) => {
                                let mut a = sm.address;
                                for byte in sm.bytes {
                                    self.cpu.memory.set_force(a, byte);
                                    a += 1;
                                }
                            }
                            GetMemory(address) => {
                                self.cpu.memory.get(address);
                            }
                            FileModified(watched_file) => {
                                let main_memory = ! watched_file.path.ends_with("aux");
                                self.cpu.memory.load_file(&watched_file.path, watched_file.address,
                                    0, 0, main_memory);
                                println!("Loaded file {} at address ${:04X} ({} mem)",
                                    watched_file.path, watched_file.address,
                                    if main_memory { "main" } else { "aux" }
                                );
                                if let Some(pc) = watched_file.starting_address {
                                    self.cpu.pc = pc;
                                }
                            }
                            SwapDisks => {
                                self.cpu.memory.disk_controller.swap_disks();
                            }
                            Reboot => {
                                stop = true;
                                rebooting = true;
                                self.cpu.memory.on_reboot();
                            }
                            SaveGraphics => {
                                save_graphic_memory(&mut self.cpu.memory);
                            }
                            LoadDisk(drive_number, disk_info) => {
                                ui_log(&format!("Loading {} in drive {drive_number}",
                                    disk_info.path()));
                                self.cpu.memory.disk_controller.load_disk_from_file(drive_number,
                                    disk_info);
                            }
                            LockDisk(drive_number) => {
                            }
                            UnlockDisk(drive_number) => {
                            }
                            CpuState(state) => {
                                match state {
                                    CpuStateMsg::Running => { paused = false; }
                                    CpuStateMsg::Paused => { paused = true; }
                                }
                            }
                            TraceStatus(trace_status) => {
                                let mut remove = false;
                                let config = &mut self.config.config;
                                if let Some(debug_asm) = trace_status.debug_asm {
                                    config.debug_asm = debug_asm;
                                }
                                if let Some(csv) = trace_status.csv {
                                    config.csv = csv;
                                }
                                if let Some(trace_to_file) = trace_status.trace_file {
                                    if trace_to_file { remove = true; }
                                    config.trace_to_file = trace_to_file;
                                    config.debug_asm = trace_to_file;
                                }
                                if let Some(trace_file_csv) = trace_status.trace_file_csv {
                                    config.trace_file_csv = trace_file_csv;
                                }
                                if let Some(trace_file_asm) = trace_status.trace_file_asm {
                                    config.trace_file_asm = trace_file_asm;
                                }
                                match &self.handle {
                                    None => {}
                                    Some(h) => {
                                        h.set_config(configure_log(config, remove));
                                    }
                                }
                            }
                            GenerateDisassembly(d) => {
                                d.generate(&self.cpu.memory.memories[0], &self.cpu.operands);
                            }
                            Debug => {
                                if let Some(disk) = self.cpu.memory.disk_controller.left_disk() {
                                    for i in (0..20).step_by(4) {
                                        // disk.bit_streams.dump(i);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            /*
            you can use checked_duration_since to avoid calculating new duration yourself
            const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);
            let timer = Instant::now();
            // take inputs, simulate cpu, show outputs
            if let Some(left) = timer.elapsed().checked_duration_since(FRAME) {
                sleep(left);
            }
            */
            // Complete this slice
            let now = Instant::now().duration_since(start).as_millis();
            if ! paused && ! stop && now >= next_cpu_run {
                let (st, cy) = self.steps(false);
                stop = st;
                total_cycles += cy;
                let now2 = Instant::now().duration_since(start).as_millis();
                let elapsed = (now2 - now) as u64;
                let emulator_period_ms = self.emulator_period_ms();
                if elapsed < emulator_period_ms {
                    let next = (emulator_period_ms - elapsed) as u32;
                    next_cpu_run = now2 + next as u128;
                }
            }
            // If the slice is over, reset
            // if true {
            //     let elapsed = self.last_cpu_run.elapsed().as_millis() as u32;
            //     if self.this_run_cycles >= EMULATOR_PERIOD_CYCLES && elapsed >= EMULATOR_PERIOD_MS {
            //         self.this_run_cycles = 0;
            //         self.last_cpu_run = Instant::now();
            //     }
            // } else {
            //     self.this_run_cycles = 0;
            //     self.last_cpu_run = Instant::now();
            // }

            if ! paused && ! stop {
                // Send CPU speed update
                if self.last_speed_sent.elapsed().as_millis() > 1000 {
                    send_message!(&self.sender, EmulatorSpeed(self.rolling_times.average()));
                    self.last_speed_sent = Instant::now();
                }

                // Send CPU update if the time has come
                let elapsed = self.last_memory_sent.elapsed().as_millis();
                let mut extra_text_memory: Vec<u8> = Vec::new();
                for i in 0..0x400 {
                    extra_text_memory.push(self.cpu.memory.memories[1][i + 0x400]);
                }
                if elapsed > 10 {
                    // println!("Sending CPU {elapsed}");
                    let cpu_dump = CpuDumpMsg {
                        memory: self.cpu.memory.main_memory(),
                        aux_memory: self.cpu.memory.aux_memory(),
                        a: self.cpu.a,
                        x: self.cpu.x,
                        y: self.cpu.y,
                        pc: self.cpu.pc,
                        p: self.cpu.p,
                        s: self.cpu.s,
                    };
                    let message = ToUi::CpuDump(cpu_dump);
                    send_message!(&self.sender, message);
                    self.last_memory_sent = Instant::now();
                } else {
                    // println!("Too early, not sending CPU {elapsed}");
                }
            }
        }
        rebooting
    }
}

fn save_graphic_memory(m: &mut Apple2Memory) {
    let mut count = 0;
    let mut start = 0x2000;
    for filename in ["pic.hgr.aux", "pic.hgr"] {
        let mut buffer: Vec<u8> = Vec::new();
        while count < 0x2000 {
            buffer.push(m.get(start + count));
            count += 1;
        }
        start += 0x2000;
        count = 0;
        let full_path = format!("d:\\t\\{}", filename);
        println!("Saving in {} buffer: {} {:02X} {:02X}", full_path, buffer.len(), buffer[0], buffer[1]);
        let file = File::create(full_path);
        file.unwrap().write_all(&buffer).unwrap();
    }
}
