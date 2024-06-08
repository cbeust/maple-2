
use crossbeam::channel::*;
use std::time::Instant;
use cpu::config::Config;
use cpu::cpu::RunStatus;
use cpu::memory::Memory;
use crate::apple2_cpu::EmulatorConfigMsg;
use crate::create_apple2;
use crate::memory::Apple2Memory;
use crate::messages::{ToCpu, ToUi};

// #[test]
fn test_cycle_count() {
    let a = 4;
    let y = 0xff;
    let x = 0xc3;

    // 0000  A9 04                      LDA #$04   2
    // 0002               L2
    // 0002  A0 C3                      LDY #$C3   2
    // 0004               L1
    // 0004  A2 FF                      LDX #$FF   2
    // 0006               L0
    // 0006  CA                         DEX        2
    // 0007  D0 FD                      BNE $0006  3
    // 0009  88                         DEY        2
    // 000A  D0 F8                      BNE $0004  3
    // 000C  AA                         TAX
    // 000D  CA                         DEX
    // 000E  8A                         TXA
    // 000F  D0 F1                      BNE $0002
    let program = [0xa9, a, 0xA0, y, 0xA2, x, 0xCA, 0xD0, 0xFD, 0x88, 0xd0, 0xf8, 0xaa, 0xca, 0x8a,
        0xd0, 0xf1, 0x60 ];

    let (sender, _): (Sender<ToUi>, Receiver<ToUi>) = unbounded();
    let (_, receiver2): (Sender<ToCpu>, Receiver<ToCpu>) = unbounded();
    let mut apple2 = create_apple2::<Apple2Memory>(Some(sender), None, Some(receiver2), [None, None],
        EmulatorConfigMsg::default(), None);

    let p = program;
    for i in 0..p.len() {
        apple2.cpu.cpu.memory.set(i as u16, p[i]);
    }
    apple2.cpu.cpu.pc = 0;
    let start = Instant::now();
    let mut steps = 0;
    let config = Config::default();
    let mut status = apple2.cpu.cpu.step(&config);
    let mut total_cycles: u128 = 0;
    while matches!(status, RunStatus::Continue(_)) {
        match status {
            RunStatus::Continue(cycles) => total_cycles += cycles as u128,
            _ => {}
        }
        status = apple2.cpu.cpu.step(&config);
        steps += 1;
        if apple2.cpu.cpu.pc == (p.len() - 1) as u16 {
            status = RunStatus::Stop(true, "Normal stop".into(), 0);
        }
    }
    let elapsed = start.elapsed().as_millis();
    assert_eq!(total_cycles, 1000659);
    let mhz = total_cycles / elapsed / 1000;
    // println!("Emulator has stopped, total cycles: {}, time: {} ms, {} Mhz",
    //          total_cycles, elapsed, mhz);
}


// fn test_cycle_count2() {
//     let program2 = [0xa2, 0xa, 0xa9, 197, 0x20, 0xa8, 0xfc, 0xca, 0xd0, 0xf8, 0x60];
// }