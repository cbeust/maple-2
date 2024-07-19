
use std::fs::File;
use std::io::Read;
use crate::disassembly::{Disassemble, DisassemblyLine};
use crate::operand::Operand;

pub trait Memory {
    fn get(&mut self, address: u16) -> u8;
    fn set(&mut self, address: u16, value: u8);
    fn set_force(&mut self, address: u16, value: u8);
    fn main_memory(&mut self) -> Vec<u8>;

    /// The address can wrap around page zero (e.g. $ff then $00) so need
    /// to get the individual bytes one by one
    fn word_ind_y(&mut self, address: u16, ind_y: bool) -> u16 {
        let address_plus_one = if address == 0xff && ind_y { 0_u16 }
            else { address.wrapping_add(1) };
        self.get(address) as u16 | ((self.get(address_plus_one) as u16) << 8)
    }

    fn word(&mut self, address: u16) -> u16 {
        self.get(address) as u16 | ((self.get(address.wrapping_add(1)) as u16) << 8)
    }

    fn disassemble(&mut self, operands: &[Operand], address: u16) -> DisassemblyLine {
        Disassemble::disassemble(&self.main_memory(), operands, address)
    }
}

pub struct DefaultMemory {
    buffer: Vec<u8>,
}

impl Memory for DefaultMemory {
    fn get(&mut self, index: u16) -> u8 {
        self.buffer[index as usize]
    }

    fn set(&mut self, index: u16, value: u8) {
        self.buffer[index as usize] = value;
    }

    fn set_force(&mut self, index: u16, value: u8) { self.buffer[index as usize] = value; }

    fn main_memory(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for i in &self.buffer { result.push(*i) }
        result
    }
}

impl DefaultMemory {
    pub const MEMORY_SIZE: u32 = 65_536;

    pub fn new() -> DefaultMemory {
        let mut buffer = Vec::new();
        for _ in 0..=DefaultMemory::MEMORY_SIZE {
            buffer.push(0);
        }
        DefaultMemory {
            buffer,
        }
    }

    pub fn new_with_file(file_name: &str) -> DefaultMemory {
        let mut f = File::open(file_name).expect("Couldn't find the file");
        let mut buffer = [0; DefaultMemory::MEMORY_SIZE as usize].to_vec();
        let _ = f.read(&mut buffer).expect("Could not read the file");

        DefaultMemory {
            buffer,
        }
    }
}


//
//
// fn _word(buffer: &Vec<u8>, index: usize) -> u16 {
//     return buffer[index + 1] as u16 | ((buffer[index + 2] as u16) << 8);
// }
//
// pub fn word2(b0: u8, b1: u8) -> u16 {
//     return b0 as u16 | ((b1 as u16) << 8);
// }
