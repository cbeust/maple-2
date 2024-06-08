#[derive(PartialEq, Debug, Copy, Clone)]

#[allow(non_camel_case_types)]
pub enum AddressingType {
    Immediate, Zp, Zp_X, Zp_Y, Absolute, Absolute_X, Absolute_Y, Indirect_X, Indirect_Y, Register_A,
    Indirect, Relative, Zpi, Zp_Relative, Indirect_Abs_X, Unknown
}

fn h(v: u8) -> String { format!("{:02X}", v) }

fn hh(v: u16) -> String { format!("{:04X}", v) }

use std::fmt::{Display, Formatter, Result};
use crate::cpu::Cpu;
use crate::memory::Memory;

impl Display for AddressingType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl AddressingType {
    pub fn to_string(&self, pc: u16, byte: u8, word: u16) -> String {
        use AddressingType::*;
        match self {
            Immediate => format!("#${}", h(byte)),
            Zp => format!("${}", h(byte)),
            Zp_X => format!("${},X", h(byte)),
            Zp_Y => format!("${},Y", h(byte)),
            Zpi => format!("(${})", h(byte)),
            Absolute => format!("${}", hh(word)),
            Absolute_X => format!("${},X", hh(word)),
            Absolute_Y => format!("${},Y", hh(word)),
            Indirect_X => format!("(${},X)", h(byte)),
            Indirect_Y => format!("(${}),Y", h(byte)),
            Indirect => format!("$({})", hh(word)),
            Indirect_Abs_X => {
                format!("(${},X)", hh(word))
            },
            Zp_Relative => {
                let byte = (word >> 8) & 0xff;
                let mut new_pc = pc.wrapping_add(byte) + 3;
                if byte >= 0x80 {
                    new_pc = (new_pc as i64 - 0x100) as u16;
                }
                format!("${}", hh(new_pc))
            }
            Relative => {
                let signed: i64 = 2_i64 + pc as i64 + byte as i64;
                let subtract: i64 = if byte >= 0x7f {0x100} else {0};
                let value = (signed - subtract) as u16;
                format!("${}", hh(value))
            },
            _ => "".to_string()
        }
    }

    // Only used by JMP
    //     fn deref16(&self, mut memory: Memory, pc: usize) -> u16 {
    //         let w = memory.word(pc.wrapping_add(1)) as usize;
    //         memory.word(w)
    //     }

    pub(crate) fn address<T: Memory>(&self, pc: u16, cpu: &mut Cpu<T>) -> u16 {
        use AddressingType::*;

        let memory = &mut cpu.memory;
        fn zp(a: u8, b: u8) -> u8 {
            (a as u16 + b as u16) as u8
        }
        match self {
            Zp => memory.get(pc.wrapping_add(1)) as u16,
            Zp_X => zp(memory.get(pc.wrapping_add(1)), cpu.x) as u16,
            Zp_Y => zp(memory.get(pc.wrapping_add(1)), cpu.y) as u16,
            Absolute => memory.word(pc.wrapping_add(1)),
            Absolute_X => (memory.word(pc.wrapping_add(1)) as u32 + cpu.x as u32) as u16,
            Absolute_Y => (memory.word(pc.wrapping_add(1)) as u32 + cpu.y as u32) as u16,
            Indirect => {
                let address = memory.word(pc.wrapping_add(1));
                let next_address =
                    if cpu.is_65c02 {
                        // For the 65C02, always jump to the next byte even if at end of page
                        address.wrapping_add(1)
                    } else {
                        // For 6502 only:
                        // Fix test "6c ff 70"
                        // AN INDIRECT JUMP MUST NEVER USE A VECTOR BEGINNING ON THE LAST BYTE
                        // OF A PAGE
                        // For example if address $3000 contains $40, $30FF contains $80, and $3100
                        // contains $50, the result of JMP ($30FF) will be a transfer of control to
                        // $4080 rather than $5080 as you intended i.e. the 6502 took the low byte of
                        // the address from $30FF and the high byte from $3000.
                        if address & 0xff == 0xff {
                            address & 0xff00
                        } else {
                           address.wrapping_add(1)
                        }
                    };
                (memory.get(next_address) as u16) << 8 | memory.get(address) as u16
            }
            Indirect_X => {
                // The address can wrap around page zero (e.g. $ff then $00) so need
                // to get the individual bytes one by one
                let v0 = memory.get(pc.wrapping_add(1));
                let byte0 = memory.get(zp(v0, cpu.x) as u16);
                let byte1 = memory.get(zp(v0, cpu.x.wrapping_add(1)) as u16);
                ((byte1 as u16) << 8) | (byte0 as u16)
            },
            Indirect_Y => {
                let zp = memory.get(pc.wrapping_add(1));
                let word = memory.word_ind_y(zp as u16, true);
                word.wrapping_add(cpu.y as u16)
            },
            Indirect_Abs_X => {
                let word = memory.word(pc.wrapping_add(1));
                let content = word.wrapping_add(cpu.x as u16);
                memory.word(content)
            }
            Zpi => {
                let zp = memory.get(pc.wrapping_add(1)) as u16;
                let byte0 = memory.get(zp);
                let byte1 = memory.get(if zp == 0xff { 0 } else { zp + 1 }) as u16;
                (byte1 << 8) | (byte0 as u16)
            }
            Zp_Relative => {
                memory.get(pc.wrapping_add(1)) as u16
            }
            Immediate | Relative | Register_A | Unknown => 0

        }
    }
}

