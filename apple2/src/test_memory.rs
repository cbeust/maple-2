use cpu::memory::Memory;
use crate::memory::*;
use crate::memory_constants::*;
use crate::{clear_soft_switch, set_soft_switch};
use crate::roms::RomType;

// #[test]
pub fn test_set_and_reset_switches() {
    let mut m = Apple2Memory::new([None, None], [None, None], None);
    struct Test {
        on: u16,
        off: u16,
        status: u16,
    }
    fn c(on: u16, off: u16, status: u16) -> Test {
        Test { on, off, status }
    }
    let tests = vec![
        c(EIGHTY_STORE_ON, EIGHTY_STORE_OFF, EIGHTY_STORE_STATUS),
        c(TEXT_ON, TEXT_OFF, TEXT_STATUS),
        c(HIRES_ON, HIRES_OFF, HIRES_STATUS),
        c(MIXED_ON, MIXED_OFF, MIXED_STATUS),
        c(PAGE_2_ON, PAGE_2_OFF, PAGE_2_STATUS),
        c(READ_AUX_MEM_ON, READ_AUX_MEM_OFF, READ_AUX_MEM_STATUS),
        c(WRITE_AUX_MEM_ON, WRITE_AUX_MEM_OFF, WRITE_AUX_MEM_STATUS),
        c(INTERNAL_CX_ON, INTERNAL_CX_OFF, INTERNAL_CX_STATUS),
        c(ALT_ZP_ON, ALT_ZP_OFF, ALT_ZP_STATUS),
        c(ALT_CHAR_ON, ALT_CHAR_OFF, ALT_CHAR_STATUS),
        c(SLOT_C3_ON, SLOT_C3_OFF, SLOT_C3_STATUS),
        c(EIGHTY_COLUMNS_ON, EIGHTY_COLUMNS_OFF, EIGHTY_COLUMNS_STATUS),
    ];
    for test in tests {
        assert_eq!(m.get(test.status) >= 0x80, false, "Expected default switch off: {:04X}", test.status);
        m.set(test.on, 0x80);
        assert_eq!(m.get(test.status) >= 0x80, true, "Expected status to be on: {:04X}", test.status);
        m.set(test.off, 0x80);
        assert_eq!(m.get(test.status) >= 0x80, false, "Expected status to be off: {:04X}", test.status);
        log(&format!("Passed status for {:04X}", test.status));
    }
}

// #[test]
fn log(s: &str) {
    println!("{s}");
}

// #[test]
pub const D: usize = 0xd1cb;
// #[test]
pub const F: usize = 0xfe1f;

// #[test]
pub fn test_high_ram() {
    struct Test {
        // true for read, false for write
        addresses: Vec<(u16, bool)>,
        expected: [u8; 5],
    }

    fn c(addresses: Vec<(u16, bool)>, expected: [u8; 5]) -> Test {
        Test {
            addresses, expected
        }
    }

    // Default state:  $53 $60 $11 $22 $33
    let tests: Vec<Test> = vec![
        c(vec![(0xc088, true)], [ 0x11, 0x33, 0x11, 0x22, 0x33 ]),
        c(vec![(0xc080, true)], [ 0x22, 0x33, 0x11, 0x22, 0x33 ]),
        c(vec![(0xc081, true)], [ 0x53, 0x60, 0x11, 0x22, 0x33 ]),
        c(vec![(0xc081, true), (0xc089, true)], [0x53, 0x60, 0x54, 0x22, 0x61]),
        c(vec![(0xc081, true), (0xc081, true)], [0x53, 0x60, 0x11, 0x54, 0x61]),
        c(vec![(0xc081, true), (0xc081, true), (0xc081, false)], [0x53, 0x60, 0x11, 0x54, 0x61]),
        c(vec![(0xc081, true), (0xc081, true), (0xc081, true), (0xc081, true),],
            [0x53, 0x60, 0x11, 0x54, 0x61]),
        c(vec![(0xc08b, true)], [0x11, 0x33, 0x11, 0x22, 0x33]),
        c(vec![(0xc083, true)], [0x22, 0x33, 0x11, 0x22, 0x33]),
        c(vec![(0xc08b, true), (0xc08b, true)], [0x12, 0x34, 0x12, 0x22, 0x34]),
        c(vec![(0xc08f, true), (0xc087, true)], [0x23, 0x34, 0x11, 0x23, 0x34]),
        c(vec![(0xc087, true), (0xc08d, true)], [0x53, 0x60, 0x54, 0x22, 0x61]),
        c(vec![(0xc08b, true), (0xc08b, false), (0xc08b, true)], [0x11, 0x33, 0x11, 0x22, 0x33]),
        c(vec![(0xc08b, false), (0xc08b, false), (0xc08b, true)], [0x11, 0x33, 0x11, 0x22, 0x33]),
        c(vec![(0xc083, true), (0xc083, true)], [0x23, 0x34, 0x11, 0x23, 0x34]),
        c(vec![(0xc083, true), (0xc083, true)], [0x23, 0x34, 0x11, 0x23, 0x34]),
    ];

    for (index, test) in tests.iter().enumerate() {
        let mut m = {
            let mut m = Apple2Memory::new([None, None], [None, None], None);
            m.memories[0][D] = 0x53;
            m.memories[0][F] = 0x60;
            m.high_ram[0].banks[0][D - 0xd000] = 0x11;
            m.high_ram[0].banks[1][D - 0xd000] = 0x22;
            m.high_ram[0].high_ram[F - 0xe000] = 0x33;
            m
        };

        for a in &test.addresses {
            if a.1 {
                let _ = m.get(a.0);
            } else {
                m.set(a.0, 0);
            }
        }

        let old = m.get(D as u16);
        m.set(D as u16, old + 1);
        let old2 = m.get(F as u16);
        m.set(F as u16, old2 + 1);

        let expected = &test.expected;
        let mut i = 0;
        let v = m.get(D as u16);
        assert_eq!(v, expected[i], "Test {}: {:04X} error, expected {:02X}, got {:02X}",
            index, D, expected[i], v);
        i += 1;
        let v = m.get(F as u16);
        assert_eq!(v, expected[i], "Test {}: {:04X} error, expected {:02X}, got {:02X}",
            index, D, expected[i], v);
        i += 1;
        let v = m.high_ram[0].banks[0][D - 0xd000];
        assert_eq!(v, expected[i], "Test {}: Bank 1 error, expected {:02X}, got {:02X}",
            index, expected[i], v);
        i += 1;
        let v = m.high_ram[0].banks[1][D - 0xd000];
        assert_eq!(v, expected[i], "Test {}: Bank 2 error, expected {:02X}, got {:02X}",
            index, expected[i], v);
        i += 1;
        let v = m.high_ram[0].high_ram[F - 0xe000];
        assert_eq!(v, expected[i], "Test {}: High ram error, expected {:02X}, got {:02X}",
            index, expected[i], v);
    }
    log(&format!("Passed {} high ram tests", tests.len()));
}

// #[test]
pub fn test_lang_card() {
    let mut m = Apple2Memory::new([None, None], [None, None], None);
    m.get(0xc08b);
    m.get(0xc08b);
    m.set(D as u16, 0x44);

    let v = m.get(D as u16);
    assert_eq!(v, 0x44, "{:04X} should be 0x44 but got {:02X}", D, v);
}

// #[test]
pub fn test_aux_mem() {
    fn aux_memory(m: &mut Apple2Memory) {
        // println! ("= Switching to aux memory");
        set_soft_switch!(m, ALT_ZP_STATUS);
        set_soft_switch!(m, READ_AUX_MEM_STATUS);
        set_soft_switch!(m, WRITE_AUX_MEM_STATUS);
    }

    fn main_memory(m: &mut Apple2Memory) {
        // println! ("= Switching to main memory");
        clear_soft_switch!(m, ALT_ZP_STATUS);
        clear_soft_switch!(m, READ_AUX_MEM_STATUS);
        clear_soft_switch!(m, WRITE_AUX_MEM_STATUS);
    }

    fn reset_all(m: &mut Apple2Memory) {
        clear_soft_switch!(m, READ_AUX_MEM_STATUS);
        clear_soft_switch!(m, WRITE_AUX_MEM_STATUS);
        clear_soft_switch!(m, EIGHTY_STORE_STATUS);
        clear_soft_switch!(m, INTERNAL_CX_STATUS);
        clear_soft_switch!(m, ALT_ZP_STATUS);
        clear_soft_switch!(m, SLOT_C3_STATUS);
        m.set(INTERNAL_C8_ROM_OFF, 0);
        clear_soft_switch!(m, EIGHTY_COLUMNS_STATUS);
        clear_soft_switch!(m, ALT_CHAR_STATUS);
        clear_soft_switch!(m, TEXT_STATUS);
        clear_soft_switch!(m, MIXED_STATUS);
        clear_soft_switch!(m, PAGE_2_STATUS);
        clear_soft_switch!(m, HIRES_STATUS);
    }

    let locations: Vec<Vec<u16>> = vec![
        vec![0xff, 0x100], // zero page
        vec![0x200, 0x3ff, 0x800, 0x1fff, 0x4000, 0x4fff, 0x5fff, 0xbfff], // main memory
        vec![0x427, 0x7ff], // text
        vec![0x2000, 0x3fff], // hires
    ];

    // (location, wanted byte)
    let cx_test_data = vec![
        // C800-CFFE
        (0xc800, 0x4c), (0xca21, 0x8d), (0xcc43, 0xf0),(0xceb5, 0x7b),
        // C100-C2FF
        (0xc14d, 0xa5), (0xc16c, 0x2a), (0xc2b5, 0xad), (0xc2ff, 0x00),
        // C400-C7FF
        (0xc436, 0x8d), (0xc548, 0x18), (0xc680, 0x8b), (0xc76e, 0xcb),
        // C300-C3FF
        (0xc300, 0x2c), (0xc30a, 0x0c), (0xc32b, 0x04), (0xc3e2, 0xed),
    ];


    fn create_mem() -> Apple2Memory {
        // Initialize aux to $3 and main to $1
        let mut m = Apple2Memory::new([None, None], [None, None], None);
        m.load_roms(RomType::Apple2Enhanced);
        m
    }

    fn init(m: &mut Apple2Memory, locations: &Vec<Vec<u16>>) {
        for page in locations {
            for l in page {
                aux_memory(m);
                // println!("Setting {:04x} to 3", l);
                m.set(*l, 3);

                // Main memory
                main_memory(m);

                // println!("Setting {:04x} to 1", l);
                m.set(*l, 1);
            }
        }
    }

    fn inc(m: &mut Apple2Memory, locations: &Vec<Vec<u16>>) {
        // Increment all the memory locations
        for page in locations {
            for l in page {
                let old = m.get(*l);
                m.set(*l, old + 1);
            }
        }
    }

    let cx_skip = [255, 255, 255, 255];
    struct Test {
        number: u8,
        // 0x80 if we are ignoring the CXX page, otherwise the fingerprint we expect
        cx: [u8; 4],
        addresses: Vec<u16>,
        expected: [u8; 8],
    }

    let cx = |number: u8, cx: [u8; 4], addresses: Vec<u16>, expected: [u8; 8]| -> Test {
        Test {
            number, cx, addresses, expected
        }
    };

    let c = |number: u8, addresses: Vec<u16>, expected: [u8; 8]| -> Test {
        Test {
            number, cx: cx_skip, addresses, expected
        }
    };

    // Indices:  [ZP, MAIN, TEXT, HIRES, ZP (aux), MAIN (aux), TEXT (aux), HIRES (aux)]
    let tests = vec![
        // Everything reset
        c(1, Vec::new(), [ 2, 2, 2, 2, 3, 3, 3, 3]),
        c(2, vec!(WRITE_AUX_MEM_ON), [ 2, 1, 1, 1, 3, 2, 2, 2 ]),
        c(3, vec!(READ_AUX_MEM_ON), [ 2, 4, 4, 4, 3, 3, 3, 3 ]),
        c(4, vec!(READ_AUX_MEM_ON, WRITE_AUX_MEM_ON), [ 2, 1, 1, 1, 3, 4, 4, 4 ]),

        // Basic tests with 80_STORE_ON
        c(5, vec!(EIGHTY_STORE_ON), [ 2, 2, 2, 2, 3, 3, 3, 3 ]),
        c(6, vec!(WRITE_AUX_MEM_ON, EIGHTY_STORE_ON), [ 2, 1, 2, 1, 3, 2, 3, 2 ]),
        c(7, vec!(READ_AUX_MEM_ON, EIGHTY_STORE_ON), [ 2, 4, 2, 4, 3, 3, 3, 3 ]),
        c(8, vec!(READ_AUX_MEM_ON, WRITE_AUX_MEM_ON, EIGHTY_STORE_ON), [ 2, 1, 2, 1,    3, 4, 3, 4 ]),

        // Our four basic tests, but with 80STORE and PAGE2 ON -------
        // (400-7ff is pointing at aux mem)
        c(9, vec!(EIGHTY_STORE_ON, PAGE_2_ON), [ 2, 2, 1, 2, 3, 3, 4, 3 ]),
        c(10, vec!(WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, PAGE_2_ON), [ 2, 1, 1, 1, 3, 2, 4, 2 ]),
        c(11, vec!(EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON), [ 2, 2, 1, 1, 3, 3, 4, 4 ]),
        c(12, vec!(WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON), [ 2, 1, 1, 1, 3, 2, 4, 4 ]),
        c(13, vec!(READ_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON), [ 2, 4, 1, 1, 3, 3, 4, 4 ]),
        c(14, vec!(READ_AUX_MEM_ON, WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON),[ 2, 1, 1, 1, 3, 4, 4, 4 ]),
        c(15, vec!(READ_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON,),[ 2, 4, 2, 2, 3, 3, 3, 3 ]),
        c(16, vec!(READ_AUX_MEM_ON, WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON,),[ 2, 1, 2, 2, 3, 4, 3, 3 ]),
        c(17, vec!(EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON,),[ 2, 2, 1, 1, 3, 3, 4, 4 ]),
        c(18, vec!(WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON,), [ 2, 1, 1, 1, 3, 2, 4, 4 ]),
        c(19, vec!(READ_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON,), [ 2, 4, 1, 1, 3, 3, 4, 4 ]),
        c(20, vec!(READ_AUX_MEM_ON, WRITE_AUX_MEM_ON, EIGHTY_STORE_ON, HIRES_ON, PAGE_2_ON,), [ 2, 1, 1, 1, 3, 4, 4, 4 ]),

        // $CXXX tests
        // Banks in that order: C8-CF, C1-C2, C4-C7, C3
        cx(21, [0, 0, 0, 1], vec![], [2, 2, 2, 2, 3, 3, 3, 3 ]),
        cx(22, [0, 0, 0, 0], vec!(SLOT_C3_ON), [2, 2, 2, 2, 3, 3, 3, 3 ]),
        cx(23, [1, 1, 1, 1], vec!(INTERNAL_CX_ON), [2, 2, 2, 2, 3, 3, 3, 3 ]),
        cx(24, [1, 1, 1, 1], vec!(SLOT_C3_ON, INTERNAL_CX_ON), [2, 2, 2, 2, 3, 3, 3, 3]),
        cx(25, [1, 0, 0, 1], vec!(INTERNAL_C8_ROM_ON), [2, 2, 2, 2, 3, 3, 3, 3]),
        cx(26, [1, 0, 0, 0], vec!(INTERNAL_C8_ROM_ON, SLOT_C3_ON), [2, 2, 2, 2, 3, 3, 3, 3]),
        cx(27, [0, 0, 0, 0], vec!(INTERNAL_C8_ROM_ON, SLOT_C3_ON, INTERNAL_C8_ROM_OFF), [2, 2, 2, 2, 3, 3, 3, 3]),
        cx(28, [1, 1, 1, 1], vec!(INTERNAL_C8_ROM_ON, INTERNAL_CX_ON, INTERNAL_C8_ROM_OFF), [2, 2, 2, 2, 3, 3, 3, 3]),
        cx(29, [0, 0, 0, 0], vec!(INTERNAL_CX_ON, SLOT_C3_ON, INTERNAL_C8_ROM_ON, INTERNAL_CX_OFF),
            [2, 2, 2, 2, 3, 3, 3, 3]),
    ];

    let mut calculate_fingerprint = |m2: &mut Apple2Memory| -> [u8; 4] {
        // C8-CF, C1-C2, C4-C7, C3
        let mut roms = [1, 1, 1, 1];
        for i in (0..cx_test_data.len()).step_by(4) {
            let mut is_match = true;
            let mut slot = 0xff;
            for j in 0..4 {
                let (address, wanted) = cx_test_data[i + j];
                if slot == 0xff {
                    slot = (address & 0x0f00) >> 8;
                }
                if m2.get(address) != wanted {
                    // println!("Found mismatch for {:04X}:{:02X} wanted {:02X}", address, m2.get(address), wanted);
                    is_match = false;
                }
            }
            if ! is_match { roms[i / 4] = 0 };
            log(&format!("      Slot {}: {}", slot, if is_match { "ROM" } else { "AUX" }));
        }
        log(&format!("      Fingerprint: C8:{} C1:{} C4:{} C3:{}", roms[0], roms[1], roms[2], roms[3]));
        roms
    };

    let test_to_run = 0xff;
    for test in tests {
        if test_to_run != 0xff && test_to_run != test.number {
            continue;
        }
        let test_number = test.number;
        log(&format!("=== Starting test {}", test_number));

        let mut m = create_mem();
        reset_all(&mut m);

        log(&format!("    == Initializing test {}", test_number));
        init(&mut m, &locations);

        log(&format!("    == Configuring test {}", test_number));
        for address in test.addresses {
            m.set(address, 0);
        }

        // Test CXXX values if not skipped
        if test.cx != cx_skip {
            let f = calculate_fingerprint(&mut m);
            assert_eq!(f, test.cx, "Failed CXXX test {}, expected {:?} but got {:0?}",
                test_number, test.cx, f);
        }

        log(&format!("    == Incrementing test {}", test_number));
        inc(&mut m, &locations);

        log(&format!("    == Running test {}", test_number));
        for mul in 0..2 {
            for i in 0..4 {

                // Select the right memory
                reset_all(&mut m);
                if mul == 0 {
                    // main_memory(&mut m);
                } else {
                    aux_memory(&mut m);
                }

                // Test memory values
                let page0 = &locations[i];
                let exp = test.expected[i + mul * 4];
                for loc in page0 {
                    let v = m.get(*loc);
                    // if v != exp {
                    //     println!("Failure, 80store: {:02X}", m.memories[0][EIGHTY_STORE_STATUS as usize]);
                    //     let _ = m.get(*loc);
                    // }
                    assert_eq!(v, exp, "Failed at location {} ${:04X}, expected {} but got {} (test#{}, index {})",
                        if mul == 0 { "main" } else { "aux" }, loc, exp, v, test_number, i + mul * 4);
                }
            }
        }
        log(&format!("    == Test {} succeeded", test_number));
    }
}

