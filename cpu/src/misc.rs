use crate::memory::{DefaultMemory, Memory};
use crate::cpu::{Cpu, RunStatus };
use std::sync::mpsc::{Sender};
use crate::config::Config;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CpuMessage {
    State(CpuState),
    MemoryWrite((usize, u8))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CpuState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: usize,
    pub run_status: RunStatus,
}

impl std::fmt::Display for CpuState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let registers = std::format!("PC={:04x} A={:02X} X={:02X} Y={:02X}",
                                     self.pc, self.a, self.x, self.y);
        write!(f, "{}", registers)
    }
}

// impl <T: Memory> CpuListener<T> for Listener {
//     fn on_pc_changed(&mut self, cpu: &mut Cpu<T>, last_cycles: u8) -> RunStatus {
//         let result =
//             if cpu.pc == 0x346c || cpu.pc == 0x3469 {
//                 RunStatus::Stop(true, String::from("All tests passed"), cpu.cycles)
//             } else {
//                 if self.previous_pc != 0 && self.previous_pc == cpu.pc {
//                     let display = cpu.display();
//                     RunStatus::Stop(false,
//                                     format!("Infinite loop at PC={:2X} cycles={:04X} {}",
//                                             cpu.pc, cpu.cycles, display),
//                                     cpu.cycles)
//                 } else {
//                     self.previous_pc = cpu.pc;
//                     RunStatus::Continue(last_cycles)
//                 }
//             };
//         result
//     }
// }

pub(crate) fn create_cpu<T: Memory>(pc: u16, m: T) -> Cpu<T> {
    // let memory_listener = MListener::new();
    // let boxed = Box::new(memory_listener);
    // let m = DefaultMemory::new_with_file("6502_functional_test.bin");
    let mut result = Cpu::new(m, None, Config::default());
    result.pc = pc;
    result
}

pub(crate) fn run_emulator<T: Memory>(_: Sender<CpuMessage>, debug_asm: bool) -> RunStatus {
    let m = DefaultMemory::new_with_file("6502_functional_test.bin");
    create_cpu(0x400, m).run(Config::default())
}

// pub fn _start_emulator() -> (Arc<RwLock<Vec<u8>>>, Receiver<CpuMessage>) {
//     let mut m: Vec<u8> = Vec::new();
//     for _ in 0..Memory::MEMORY_SIZE { m.push(0) }
//     let memory: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(m));
//     let (tx, rx) = mpsc::channel::<CpuMessage>();
//     // let mem2 = Memory::new_with_file("6502_functional_test.bin", None);
//     // let arc_mem: Arc<Mutex<Memory>> = Arc::new(Mutex::new(mem2));
//     let _ = thread::spawn(move || {
//         run_emulator::<T>(tx, false);
//     });
//     (memory, rx)
// }
