use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use crossbeam::channel::{Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use log4rs::Handle;
use cpu::config::Config;
use cpu::cpu::{Cpu, RunStatus, StopReason};
use cpu::memory::Memory;
use crate::memory::Apple2Memory;
use crate::messages::{CpuDumpMsg, CpuStateMsg, ToCpu, ToUi};
use crate::messages::ToUi::{EmulatorSpeed};
use crate::misc::increase_cycles;
use crate::rolling_times::RollingTimes;
use crate::{configure_log, send_message, ui_log};
use crate::config_file::ConfigFile;
use crate::constants::{CPU_REFRESH_MS, PC, START};
use crate::ui::iced::shared::Shared;

#[derive(Clone, Debug, Default)]
pub struct EmulatorConfigMsg {
    pub config: Config,
    pub config_file: ConfigFile,
}

impl EmulatorConfigMsg {
    pub fn new(config: Config, config_file: ConfigFile) -> Self {
        Self {
            config, config_file,
        }
    }
}

fn to_cpu_state(run_status: &RunStatus, step: bool) -> CpuStateMsg {
    match run_status {
        RunStatus::Continue(_) => {
            if step {
                CpuStateMsg::Step
            } else {
                CpuStateMsg::Running
            }
        }
        RunStatus::Stop(_, _) => {
            CpuStateMsg::Paused
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
    config: Box<EmulatorConfigMsg>,
    cycles: u128,
    start: Instant,
    started: bool,
    handle: Option<Handle>,
    previous_slice_start: u128,
}

impl AppleCpu {
    pub fn new(cpu: Cpu<Apple2Memory>, config: Box<EmulatorConfigMsg>,
        sender: Option<Sender<ToUi>>,
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
            previous_slice_start: START.get().unwrap().elapsed().as_millis(),
        }
    }

    fn log(&self, s: &str) {
        // println!("[CPU {:>4}] {}", self.this_run_cycles, s);
    }

    pub fn step(&mut self) {
        if ! self.started {
            self.start = Instant::now();
            self.started = true;
        }

        if ! matches!(self.cpu.run_status, RunStatus::Stop(_, _)) {
            self.cpu.memory.disk_controller.step();
            self.advance_cpu();
            *PC.write().unwrap() = self.cpu.pc;
            self.cpu.memory.disk_controller.step();

            self.cycles += self.cpu.run_status.cycles();

            increase_cycles(1);
        }
    }

    fn advance_cpu(&mut self) {
        if self.wait == 0 {
            self.cpu.step(&self.config.config, &self.config.config_file.breakpoints_hash);
            match self.cpu.run_status {
                RunStatus::Continue(_) => {}
                RunStatus::Stop(ref reason, _) => {
                    if *reason == StopReason::BreakpointHit {
                        ui_log(&format!("Sending message to UI: BreakpointWasHit: {:04X}",
                                        self.cpu.pc));
                        send_message!(&self.sender, ToUi::BreakpointWasHit(0));
                        Shared::set_breakpoint_was_hit(true);
                    }
                }
            };
            let cycles = self.cpu.run_status.cycles();
            self.wait = (cycles - 1) as u8;
            self.this_run_cycles += cycles as u64;
        } else {
            self.wait -= 1;
            self.cpu.run_status = RunStatus::Continue(1);
        }
    }

    fn emulator_period_cycles(&self) -> u64 {
        100_000
    }

    fn emulator_period_ms(&self) -> u64 {
        1000 * self.emulator_period_cycles() / self.config.config.emulator_speed_hz
    }

    /// Advance the emulator by emulator_period_cycles() cycles (e.g. 100_000)
    pub fn steps(&mut self, debug_asm: bool) -> u128 {
        let mut total_cycles: u128 = 0;
        let mut slice_cycles: u64 = 0;
        let mut stop = false;


        let epc = self.emulator_period_cycles();
        while ! stop && slice_cycles < epc {
            self.step();
            stop = matches!(self.cpu.run_status, RunStatus::Stop(_, _));
            // println!("Advancing {} cycles", cy);
            let cy = self.cpu.run_status.cycles();
            slice_cycles += cy as u64;
            total_cycles += cy;
        }

        total_cycles
    }

    /// Return true if we're rebooting, false if we're exiting
    pub fn run(&mut self) -> CpuStateMsg {
        use ToCpu::*;
        let mut total_cycles: u128 = 0;
        let start = *START.get().unwrap();
        let mut next_cpu_run = (Instant::now() - Duration::new(10, 0)).duration_since(start).as_millis();

        let mut status = CpuStateMsg::Running;
        while status == CpuStateMsg::Running {
            if let Some(receiver) = &self.receiver {
                while status == CpuStateMsg::Running && ! receiver.is_empty() {
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
                                let paths = &self.cpu.memory.disk_controller.disks()
                                    .map(|disk| disk.map(|di| di.path));
                                self.config.config_file.set_drive(false, 0, paths[0].clone());
                                self.config.config_file.set_drive(false, 1, paths[1].clone());
                            }
                            Reboot => {
                                status = CpuStateMsg::Rebooting;
                                // stop = true;
                                // rebooting = true;
                                self.cpu.memory.on_reboot();
                            }
                            SaveGraphics => {
                                save_graphic_memory(&mut self.cpu.memory);
                            }
                            LoadDisk(is_hard_drive, drive_number, disk_info) => {
                                ui_log(&format!("Loading {} in {} drive {drive_number}",
                                    disk_info.path(),
                                    if is_hard_drive { "hard" } else { "" }
                                ));
                                self.cpu.memory.load_disk_from_file(is_hard_drive, drive_number,
                                    disk_info);
                            }
                            LockDisk(drive_number) => {
                            }
                            UnlockDisk(drive_number) => {
                            }
                            CpuState(state) => {
                                match state {
                                    CpuStateMsg::Step => {
                                        self.cpu.step(&self.config.config, &HashSet::new());
                                        self.cpu.run_status = RunStatus::Stop(StopReason::Ok, 0);
                                        status = CpuStateMsg::Step;
                                    }
                                    CpuStateMsg::Running => {
                                        self.cpu.run_status = RunStatus::Continue(0);
                                        Shared::set_breakpoint_was_hit(false);
                                        status = CpuStateMsg::Running;
                                    }
                                    CpuStateMsg::Paused => {
                                        self.cpu.run_status = RunStatus::Stop(StopReason::Ok, 0);
                                        status = CpuStateMsg::Paused;
                                    }
                                    CpuStateMsg::Exit => {
                                        send_message!(&self.sender, ToUi::Exit);
                                        self.cpu.run_status = RunStatus::Stop(StopReason::Exit, 0);
                                        status = CpuStateMsg::Exit;
                                    }
                                    _ => {}
                                };

                                Shared::get_cpu().run_status = self.cpu.run_status;
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
            let mut slice_cycles = 0;
            if status == CpuStateMsg::Running && now >= next_cpu_run {
                let slice_start = START.get().unwrap().elapsed().as_millis();

                slice_cycles = self.steps(false);
                total_cycles += self.cpu.run_status.cycles();
                status = to_cpu_state(&self.cpu.run_status, false);
                // stop = matches!(run_status, RunStatus::Stop(_, _));
                let now2 = Instant::now().duration_since(start).as_millis();
                let elapsed = (now2 - now) as u64;
                let emulator_period_ms = self.emulator_period_ms();
                if elapsed < emulator_period_ms {
                    let next = (emulator_period_ms - elapsed) as u32;
                    next_cpu_run = now2 + next as u128;
                }

                self.rolling_times.add(self.previous_slice_start, slice_start, slice_cycles);
                self.previous_slice_start = slice_start;
            }


            let elapsed = self.last_memory_sent.elapsed().as_millis();
            if elapsed > CPU_REFRESH_MS {
                self.update_context();
                self.last_memory_sent = Instant::now();
            }

            if status == CpuStateMsg::Running {
                // Send CPU speed update
                if self.last_speed_sent.elapsed().as_millis() > 1000 {
                    send_message!(&self.sender, EmulatorSpeed(self.rolling_times.average()));
                    self.last_speed_sent = Instant::now();
                }

                // Send CPU update if the time has come
                let mut extra_text_memory: Vec<u8> = Vec::new();
                for i in 0..0x400 {
                    extra_text_memory.push(self.cpu.memory.memories[1][i + 0x400]);
                }
            }
        }

        status
    }

    fn update_context(&mut self) {
        Shared::set_cpu(create_dump_msg(&mut self.cpu));
    }
}

pub(crate) static ID: RwLock<u64> = RwLock::new(0);

fn create_dump_msg(cpu: &mut Cpu<Apple2Memory>) -> CpuDumpMsg {
    let id: u64 = *ID.read().unwrap();
    *ID.write().unwrap() = id.wrapping_add(1);
    let result = CpuDumpMsg {
        id,
        memory: cpu.memory.main_memory(),
        aux_memory: cpu.memory.aux_memory(),
        a: cpu.a,
        x: cpu.x,
        y: cpu.y,
        pc: cpu.pc,
        p: cpu.p,
        s: cpu.s,
        run_status: cpu.run_status.clone(),
    };
    result
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
