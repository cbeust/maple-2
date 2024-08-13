use rand::random;
use crate::disk::disk::Disk;
use crate::disk::drive::Drive;

// It's 1one bit every 4 cpu cycles/8 lss cycles.  One nibble is between 32 and 40 cpu cycles
// (64-80 lss cycles) usually, depending on the number of 0 sync bits.  As for the clearing
// of the latch, it depends if the first two bits (post-optional-sync) are 10 or 11.
// On 10 the latch is cleared 12 lss cycles after the first 1 (50% margin).
// On 11 3 lss cycles after the second 1.

/// Beneath Apple ProDOS - pages D-6 and D-7
const P6: [u8; 256] = [
    //                Q7 L (Read)                                         Q7 H (Write)
    //       Q6 L                     Q6 H                   Q6 L (Shift)               Q6 H (Load)
    //  QA L        QA H         QA L        QA H           QA L        QA H         QA L        QA H
    //1     0     1     0      1     0     1     0        1     0     1     0      1     0     1     0
    0x18, 0x18, 0x18, 0x18, 0x0A, 0x0A, 0x0A, 0x0A, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, // 0
    0x2D, 0x2D, 0x38, 0x38, 0x0A, 0x0A, 0x0A, 0x0A, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, 0x28, // 1
    0xD8, 0x38, 0x08, 0x28, 0x0A, 0x0A, 0x0A, 0x0A, 0x39, 0x39, 0x3b, 0x3b, 0x39, 0x39, 0x3B, 0x3B, // 2
    0xD8, 0x48, 0x48, 0x48, 0x0A, 0x0A, 0x0A, 0x0A, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, 0x48, // 3
    0xD8, 0x58, 0xD8, 0x58, 0x0A, 0x0A, 0x0A, 0x0A, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, 0x58, // 4
    0xD8, 0x68, 0xD8, 0x68, 0x0A, 0x0A, 0x0A, 0x0A, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, 0x68, // 5
    0xD8, 0x78, 0xD8, 0x78, 0x0A, 0x0A, 0x0A, 0x0A, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, 0x78, // 6
    0xD8, 0x88, 0xD8, 0x88, 0x0A, 0x0A, 0x0A, 0x0A, 0x08, 0x88, 0x08, 0x88, 0x08, 0x88, 0x08, 0x88, // 7
    0xD8, 0x98, 0xD8, 0x98, 0x0A, 0x0A, 0x0A, 0x0A, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, 0x98, // 8
    0xD8, 0x29, 0xD8, 0xA8, 0x0A, 0x0A, 0x0A, 0x0A, 0xA8, 0xA8, 0xA8, 0xA8, 0xA8, 0xA8, 0xA8, 0xA8, // 9
    0xCD, 0xBD, 0xD8, 0xB8, 0x0A, 0x0A, 0x0A, 0x0A, 0xB9, 0xB9, 0xBB, 0xBB, 0xB9, 0xB9, 0xBB, 0xBB, // A
    0xD9, 0x59, 0xD8, 0xC8, 0x0A, 0x0A, 0x0A, 0x0A, 0xC8, 0xC8, 0xC8, 0xC8, 0xC8, 0xC8, 0xC8, 0xC8, // B
    0xD9, 0xD9, 0xD8, 0xA0, 0x0A, 0x0A, 0x0A, 0x0A, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, 0xD8, // C
    0xD8, 0x08, 0xE8, 0xE8, 0x0A, 0x0A, 0x0A, 0x0A, 0xE8, 0xE8, 0xE8, 0xE8, 0xE8, 0xE8, 0xE8, 0xE8, // D
    0xFD, 0xFD, 0xF8, 0xF8, 0x0A, 0x0A, 0x0A, 0x0A, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, // E
    0xDD, 0x4D, 0xE0, 0xE0, 0x0A, 0x0A, 0x0A, 0x0A, 0x88, 0x08, 0x88, 0x08, 0x88, 0x08, 0x88, 0x08  // F
];

#[derive(Default)]
pub struct Lss {
    clock: u128,
    state: u8,
    zeros: u16,
    pub latch: u8,
}

impl Lss {
    pub(crate) fn reset(&mut self) {
        self.latch = self.latch & 0x7F;
        self.state = 0;
    }
}

impl Lss {
    pub fn on_pulse(&mut self, q6: bool, q7: bool, motor_on: bool, drive: &mut Drive) {
        let phase_160 = drive.get_phase_160();
        if let Some(d) = &mut drive.disk {
            if motor_on {
                self.step(q6, q7, motor_on, phase_160, d);
            }
        }
    }

    fn step(&mut self, q6: bool, q7: bool, motor_on: bool, phase_160: usize, disk: &mut Disk) {
        let mut pulse = 0;
        // Adding q6 and q7 tests break Algernonn
        if self.clock == 4 && ! q7 && ! q6 {
            pulse = disk.next_bit(phase_160);
            if pulse == 0 {
                // Just need to know that there were more than 2 zeros in a row, no point in saturating
                // that number
                if self.zeros < 10 { self.zeros += 1; }
                if self.zeros > 2 {
                    pulse = if random::<f32>() < 0.3 { 1 } else { 0 }
                }
            } else {
                self.zeros = 0;
            }
        }

        let mut idx = 0_u8;
        let qa = ((self.latch & 0x80) >> 7) != 0;
        // q7 is read/write switch, q6 is shift/load switch
        idx =
            if pulse == 0 { 1 } else { 0 }
            | (if qa { 2 } else { 0 })
            | (if q6 { 4 } else { 0 })
            | (if q7 { 8 } else { 0 })
            | (self.state << 4);

        // Table 9.3, page 0-16 of Understanding the Apple 2, Jim Sather
        let command = P6[idx as usize];
        match command & 0xf {
            0..=7 => {
                // CLR
                self.latch = 0;
            }
            8 | 0xc => { /* NOP */ }
            9 => {
                // SLO
                self.latch <<= 1;
            }
            0xa | 0xe => {
                // SR
                self.latch >>= 1;
                self.latch += 0x80; // This should be write protect
            }
            0xb | 0xf => {
                // LD
                // Nothing to do, already capturing that nibble when stored in $C08D
            }
            0xd => {
                // SL1
                self.latch = (self.latch << 1) | 1;
            }
            _ => {
                panic!("LSS: Should never happen");
            }
        }
        self.state = command >> 4;

        // if self.clock == 4 {
        //     if motor_on {
        //         if q7 {
        //             log::error!("LSS: Write {:02X}", self.latch);
        //             self.write_buffer.push(self.latch);
        //         }
        //     }
        // }

        self.clock = (self.clock + 1) % 8;
    }
}