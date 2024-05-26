use std::ops::Range;
use crate::constants::{CYCLES, PC};
use crate::disk::bit_stream::Nibble;

pub const DEBUG_DISASM: Option<Range<i32>> = Some(0xc65e..0xc6b8);

pub fn hex_dump_at_fn<T>(b: &[T], address: u16, length: u16, f: fn(&T) -> String) {
    for i in 0..=length {
        if i > 0 && i % 16 == 0 {
            println!();
        }
        let offset = (address + i) as usize;
        if i% 16 == 0 {
            print!("{:04X} | ", offset);
        }
        if offset < b.len() { print!("{} ", f(&b[offset])) }
    }
    println!("\n====");
}

pub fn hex_dump_at(b: &[u8], address: u16, length: u16) {
    hex_dump_at_fn(b, address, length, |s| { format!("{:02X}", s) });
}

#[allow(unused)]
pub fn hex_dump(b: &[u8]) {
    hex_dump_at_fn(b, 0_u16, b.len() as u16, |s| { format!("{:02X}", s) });
}

pub fn hex_dump_nibbles(b: &[Nibble]) {
    hex_dump_at_fn(b, 0_u16, b.len() as u16, |s| { format!("{}", s) });
}

pub fn hex_dump_fn<T>(b: &[T], f: fn(&T) -> String) {
    hex_dump_at_fn(b, 0_u16, b.len() as u16, f);
}

#[allow(unused)]
pub(crate) fn log_emulator(s: &str) {
    println!("{:8} | {:04X} | {}", *CYCLES.read().unwrap(), *PC.read().unwrap(), s);
}
