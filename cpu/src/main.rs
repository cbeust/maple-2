use std::time::Instant;
use cpu::cpu::RunStatus;

fn main() {
    // let mut cpu = crate::create_cpu(0x400);
    // let start = Instant::now();
    // let mut total_cycles: u128 = 0;
    // let mut status = cpu.step();
    // while matches!(status, RunStatus::Continue(_)) {
    //     match status {
    //         RunStatus::Continue(cycles) => total_cycles += cycles as u128,
    //         _ => {}
    //     }
    //     status = cpu.step();
    // }
    // let elapsed = start.elapsed().as_millis();
    // let mhz = total_cycles / elapsed / 1000;
    // println!("Emulator has stopped, total cycles: {}, time: {} ms, {} Mhz",
    //         total_cycles, elapsed, mhz);
}
