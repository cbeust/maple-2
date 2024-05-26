#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::config::Config;
    use crate::constants::OPERANDS_6502;
    use crate::memory::{DefaultMemory, Memory};
    use crate::cpu::{Cpu, RunStatus};
    use crate::cpu::RunStatus::Continue;

    #[test]
    fn test_functional_6502() {
        let (passed, reason) = run_functional_tests("../6502_functional_test.bin", false, 0x346c);
        assert!(passed, "{}", reason)
    }

    #[test]
    pub fn test_functional_65c02() {
        let (passed, reason) = run_functional_tests("../65C02_extended_opcodes_test.bin", true, 0x24f1);
        assert!(passed, "{}", reason)
    }

    fn run_functional_tests(file: &str, is_65c02: bool, success_pc: u16) -> (bool, String) {
        let config = Config {
            debug_asm: true,
            is_65c02,
            ..Default::default()
        };
        let m = DefaultMemory::new_with_file(file);
        let now = Instant::now();
        let mut cpu = Cpu::new(m, config.clone());
        let mut cycles = 128;
        cpu.pc = 0x400;
        let mut previous_pc = 0;

        fn success(now: Instant, cycles: u128) -> String {
            let elapsed = now.elapsed().as_millis();
            let mhz: f32 = (cycles as f32 / elapsed as f32 / 1000.0);
            format!("Success: ran {} cycles in {} ms, {} Mhz", cycles, elapsed, mhz)
        }

        let mut stop = false;
        let mut passed = false;
        let mut reason = "".to_string();
        while ! stop {
            let status = cpu.step(&config);
            match status {
                Continue(c) => {
                    if cpu.pc == success_pc { // } || cpu.pc == 0x3469 {
                        stop = true;
                        passed = true;
                        reason = success(now, cycles);
                    } else if previous_pc != 0 && previous_pc == cpu.pc {
                        stop = true;
                        passed = false;
                        reason = format!("Infinite loop at PC={:2X} cycles={:04X} {}", cpu.pc, cpu.cycles, cpu);
                    } else {
                        previous_pc = cpu.pc;
                        cycles += c as u128;
                    }
                }
                RunStatus::Stop(s, ref _reason, cycles) => {
                    if s {
                        stop = true;
                        passed = true;
                        reason = success(now, cycles);
                    } else {
                        assert!(s, "{}", reason)
                    }
                }
            }
        };

        println!("End tests: {} {}", if passed { "Passed" } else { "FAILED" }, reason);
        (passed, reason)
    }

    #[test]
    fn sizes() {
        let sizes: [usize; 256] = [
          //0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
            1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0x00-0x0f
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0x10-0x1f
            3, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0x20-0x2f
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0x30-0x3f
            1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0x40-0x4f
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0x50-0x5f
            1, 2, 0, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0x60-0x6f
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0x70-0x7f
            2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0x80-0x8f
            2, 2, 0, 3, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 2,  // 0x90-0x9f
            2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0xa0-0xaf
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0xb0-0xbf
            2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0xc0-0xcf
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3,  // 0xd0-0xdf
            2, 2, 2, 2, 2, 2, 2, 2, 1, 2, 1, 2, 3, 3, 3, 3,  // 0xe0-0xef
            2, 2, 0, 2, 2, 2, 2, 2, 1, 3, 1, 3, 3, 3, 3, 3   // 0xf0-0xff
        ];

        for (i, op) in OPERANDS_6502.iter().enumerate() {
            // if op.size as usize != sizes[i] {
            //     println!("Sizes:   [{:02X}] = {}", i, op.size);
            // }
            assert_eq!(op.size as usize, sizes[i], "Different sizes for index ${:02x}", i);
        }
    }

    #[test]
    fn cycles() {
        /// Number of clock cycles required for each instruction when
        let cycles: [u8; 256] = [
          //0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f
            7, 6, 2, 1, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 7,  // 0x00-0x0f
            2, 5, 5, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 8,  // 0x10-0x1f
            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,  // 0x20-0x2f
            2, 5, 5, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 0x30-0x3f
            6, 6, 2, 7, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,  // 0x40-0x4f
            2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 0x50-0x5f
            6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,  // 0x60-0x6f
            2, 5, 5, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 0x70-0x7f
            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // 0x80-0x8f
            2, 6, 5, 5, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 6,  // 0x90-0x9f
            2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,  // 0xa0-0xaf
            2, 5, 5, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,  // 0xb0-0xbf
            2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // 0xc0-0xcf
            2, 5, 5, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,  // 0xd0-0xdf
            2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,  // 0xe0-0xef
            2, 5, 5, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7   // 0xf0-0xff
        ];


        for (i, op) in OPERANDS_6502.iter().enumerate() {
            // if op.cycles as usize != cycles[i] as usize {
            //     println!("Cycles [{:02X}] = {}", i, op.cycles);
            // }
            assert_eq!(op.cycles, cycles[i], "Different cycles for index ${:02x}", i);
        }
    }

}