#![allow(unused)]
#![allow(warnings)]

use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::borrow::BorrowMut;
use crate::memory::{DefaultMemory, Memory};
use crate::constants::*;

const _DEBUG_ASM: bool = true;
const DEBUG_PC: u16 = 0;
const DEBUG_CYCLES: u128 = u128::MAX; // 0x4FC1A00

const STACK_ADDRESS: u16 = 0x100;

#[derive(Default, Clone, Copy)]
pub struct StatusFlags {
    _value: u8
}

use crate::addressing_type::AddressingType::*;

use std::convert::TryInto;
use std::fmt::{Display, Formatter};
use std::process::exit;
use std::time::Instant;
use crossbeam::channel::Sender;
use crate::addressing_type::AddressingType;
use crate::config::Config;
use crate::constants;
use crate::disassembly::{Disassemble, RunDisassemblyLine};
use crate::log_file::log_file;
use crate::messages::{LogMsg, ToLogging};
use crate::messages::ToLogging::Log;
use crate::operand::Operand;

// fn init() -> [Operand; 256] {
//     let mut result: Vec<Operand> = Vec::new();
//     for i in 0..=255 {
//         result.push( Operand { opcode: i, size: SIZES[i as u16], name: OPCODE_NAMES[i as u16],
//             cycles: TIMINGS[i as u16],
//             addressing_type: ADDRESSING_TYPES[i as u16] });
//     }
//     match result.as_slice().try_into() {
//         Ok(array) => array,
//         Err(_) => panic!("Wrong size for vector"),
//     }
// }
//
// const OP: [Operand; 256] = init();

impl StatusFlags {
    pub fn new() -> Self {
        Self { _value: 0x30 /* reserved to true by default */ }
    }

    pub fn new_with(value: u8) -> Self {
        StatusFlags { _value: value }
    }

    pub fn set_value(&mut self, value: u8) {
        self._value = value | 1 << 4 | 1 << 5;  // always set the B and reserved flags
    }

    pub fn value(&self) -> u8 { self._value }

    fn get_bit(&self, bit: u8) -> bool {
        self._value & (1 << bit) != 0
    }

    fn set_bit(&mut self, f: bool, bit: u8) {
        if f { self._value |= 1 << bit }
        else { self._value &= !(1 << bit) }
    }

    pub fn n(&self) -> bool { self.get_bit(7) }
    pub fn set_n(&mut self, f: bool) { self.set_bit(f, 7) }
    pub fn v(&self) -> bool { self.get_bit(6) }
    pub fn set_v(&mut self, f: bool) { self.set_bit(f, 6) }
    pub fn reserved(&self) -> bool { true }  // reserved always true
    pub fn b(&self) -> bool { self.get_bit(4) } // b always true
    pub fn set_b(&mut self, f: bool) { self.set_bit(f, 4) }
    pub fn d(&self) -> bool { self.get_bit(3) }
    pub fn set_d(&mut self, f: bool) { self.set_bit(f, 3) }
    pub fn i(&self) -> bool { self.get_bit(2) }
    pub fn set_i(&mut self, f: bool) { self.set_bit(f, 2) }
    pub fn z(&self) -> bool { self.get_bit(1) }
    pub fn set_z(&mut self, f: bool) { self.set_bit(f, 1) }
    pub fn c(&self) -> bool { self.get_bit(0) }
    pub fn set_c(&mut self, f: bool) { self.set_bit(f, 0) }

    fn set_nz_flags(&mut self, reg: u8) {
        self.set_z(reg == 0);
        self.set_n(reg & 0x80 != 0);
    }
}

const DOT: &str = &".";

impl Display for StatusFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn s(n: &str, v: bool) -> &str {
            if v { n } else { DOT }
        }

        write!(f, "P={:02X} {{{}{}{}{}{}{}{}{}}}", self.value(),
               s("N", self.n()),
               s("V", self.v()),
               s("R", false), // it's always true but don't bother displaying it
               s("B", self.b()),
               s("D", self.d()),
               s("I", self.i()),
               s("Z", self.z()),
               s("C", self.c()))
    }
}

// pub type CpuListener<T> = Fn(Cpu<T>) -> RunStatus;

// pub trait CpuListener<T: Memory> {
//     /// return Ok() if the execution should continue and Err() if it should stop, in which
//     /// case the String will give the reason for the stop.
//     fn on_pc_changed(&mut self, cpu: &mut Cpu<T>, last_cycles: u8) -> RunStatus;
// }

pub struct Cpu<T: Memory> {
    pub memory: T,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub p: StatusFlags,
    pub s: u8,

    pub operands: [Operand; 256],

    /// If the previous instruction caused a write, this field contains
    /// the address and the value now stored at that address. Otherwise, it contains None.
    pub last_write: Option<(u16, u8)>,

    pub cycles: u128,
    last_cycles: u128,
    debug_line_count: u64,
    end_pc_reached: bool,
    in_range: bool,
    trace_in_progress: bool,
    trace_cycles_in_progress: bool,
    pub asm_always: bool,
    start: Instant,
    started: bool,
    pub(crate) is_65c02: bool,

    logging_sender: Option<Sender<ToLogging>>,
}

impl<T: Memory> Display for Cpu<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("A={:02X} X={:02X} Y={:02X} P={:02X} S={:02X} PC={:04X}",
            self.a, self.x, self.y, self.p.value(), self.s, self.pc));
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunStatus {
    // Number of cycles
    Continue(u8),
    // If bool is true, stopping with no error + reason for stopping
    // The u128 parameter is the number of cycles that were run
    Stop(bool, String, u128)
}

impl <T: Memory> Cpu<T> {
    pub fn new(mut memory: T, logging_sender: Option<Sender<ToLogging>>, config: Config) -> Cpu<T> {
        Cpu {
            memory,
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xff,
            p: StatusFlags::new(),
            cycles: 0,
            last_cycles: 0,
            last_write: None,
            debug_line_count: if let Some(c) = config.trace_count { c } else { 0 },
            end_pc_reached: false,
            in_range: false,
            trace_in_progress: false,
            trace_cycles_in_progress: false,
            asm_always: false,
            start: Instant::now(),
            started: false,
            is_65c02: config.is_65c02,
            operands: if config.is_65c02 { OPERANDS_65C02 } else { OPERANDS_6502 },
            logging_sender,
        }
    }

    pub fn step(&mut self, config: &Config) -> RunStatus {
        let previous_pc = self.pc;
        let opcode = self.memory.get(self.pc);
        let operand = &self.operands[opcode as usize];
        self.pc = self.pc.wrapping_add(operand.size as u16);
        let cycles = self.next_instruction(previous_pc, config);
        self.cycles = self.cycles + cycles as u128;

        let stop = RunStatus::Continue(cycles);
        let result = match stop {
            RunStatus::Stop(success, ref reason, cycles) => {
                println!("{}", reason.as_str());
                RunStatus::Stop(success, reason.to_string(), cycles)
            },
            _ => RunStatus::Continue(cycles),
        };

        result
    }

    pub fn run(&mut self, config: Config) -> RunStatus {
        let mut result = self.step(&config);
        let mut cont = true;
        while cont {
            match result {
                RunStatus::Continue(_) => result = self.step(&config),
                _ => cont = false,
            }
        }
        result
        // while result == RunStatus::Continue {
        //     result = self.step();
        // }
        // result

        // loop {
        //     let previous_pc = self.pc;
        //     let opcode = self.memory.get(self.pc) as u16;
        //     let operand = &OPERANDS[opcode];
        //     self.pc = (self.pc + operand.size) % 0x10000;
        //     self.cycles = self.cycles + self.next_instruction(previous_pc) as u128;
        //
        //     let stop = if let Some(l) = self.listener.borrow_mut().as_mut() {
        //         l.on_pc_changed(self)
        //     } else {
        //         RunStatus::Continue
        //     };
        //
        //     match stop {
        //         RunStatus::Stop(success, ref reason, cycles) => {
        //             result = RunStatus::Stop(success, reason.to_string(), cycles);
        //             println!("{}", reason.as_str());
        //             break;
        //         },
        //         _ => {}
        //     }
        // }
        // return result;
    }

    fn address_value(&mut self, pc: u16, addressing_type: AddressingType) -> (u16, u8) {
        let address = addressing_type.address(pc, self);
        let value = self.memory.get(address);
        (address, value)
    }

    /// Invoked for all the opcodes that have an IND_Y, ABS_X, or ABS_Y addressing mode,
    /// since these can cause an additional cycle if a page boundary is crossed
    fn run_indirect(&mut self, opcode: u8, pc: u16, address: u16,
        ind_y: u8, abs_x: u8, abs_y: u8) -> u8 {
        // cpu.p.set_nz_flags(cpu.a);
        let memory = &mut self.memory;
        let result =
            if opcode == ind_y {
                let value = memory.get(pc.wrapping_add(1)) as u16;
                let word = memory.word_ind_y(value, opcode == ind_y);
                self.page_crossed(word, address)
            } else if opcode == abs_x || opcode == abs_y {
                let word = memory.word_ind_y(pc.wrapping_add(1), opcode == ind_y);
                self.page_crossed(word, address)
            } else {
                0
            };
        result
    }

    pub fn next_instruction(&mut self, pc: u16, config: &Config) -> u8 {
        let max = 10;
        let mut i = 0;

        let opcode = self.memory.get(pc);

        #[cfg(feature = "log_prodos")]
        if opcode == 0x20 && self.memory.get(pc + 1) == 0 && self.memory.get(pc + 2) == 0xbf {
            log::info!("Calling ProDOS at ${:04X}, operation: {:02X}", pc, self.memory.get(pc + 3));
        }

        let operand = self.operands[opcode as usize];
        let mut addressing_type = operand.addressing_type;
        let mut cycles = operand.cycles;

        self.last_write = None;
        let mut resolved_address: Option<u16> = None;
        let mut resolved_value: Option<u8> = None;
        let mut resolved_read = true;

        // if OPCODES_65C02.contains(&opcode) {
        //     println!("65C02 opcode detected at PC {:04X}:{:02X}", pc, opcode);
        //     println!("");
        // }
        match opcode {
            ADC_IMM => self.adc(pc.wrapping_add(1)),
            ADC_ZP | ADC_ZP_X | ADC_ABS | ADC_ABS_X | ADC_ABS_Y | ADC_IND_X | ADC_IND_Y | ADC_ZPI_65C02 => {
                if opcode == ADC_ZPI_65C02 && ! self.is_65c02 {} else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.adc(address);
                    cycles += self.run_indirect(opcode, pc, address, ADC_IND_Y, ADC_ABS_X, ADC_ABS_Y);
                }
            },
            AND_IMM => {
                self.a = self.a & self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.a);
            },
            AND_ZP | AND_ZP_X | AND_ABS | AND_ABS_X | AND_ABS_Y | AND_IND_X | AND_IND_Y | AND_ZPI_65C02 => {
                if opcode == AND_ZPI_65C02 && ! self.is_65c02 {} else {
                    let address = self.and_op(addressing_type, pc);
                    let (_, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    if opcode != AND_ABS && opcode != AND_ZP { resolved_address = Some(address) }
                    cycles += self.run_indirect(opcode, pc, address, AND_IND_Y, AND_ABS_X, AND_ABS_Y);
                }
            },
            ASL => self.a = self.asl(self.a),
            ASL_ZP | ASL_ZP_X | ASL_ABS | ASL_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);

                let result = self.asl(value);
                self.memory.set(address, result);
                self.last_write = Some((address, result));
            },
            BIT_ZP | BIT_ABS | BIT_ZP_X_65C02 | BIT_ABS_X_65C02 => {
                if (opcode == BIT_ZP_X_65C02 || opcode == BIT_ABS_X_65C02) && ! self.is_65c02 {}
                else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.p.set_z(value & self.a == 0);
                    self.p.set_n(value & 0x80 != 0);
                    self.p.set_v(value & 0x40 != 0);
                }
            },
            BIT_IMM_65C02 if self.is_65c02 => {
                let value = self.memory.get(pc.wrapping_add(1));
                self.p.set_z(value & self.a == 0);
            }
            BRA_65C02 if self.is_65c02 => { cycles += self.branch(pc.wrapping_add(1), true) }
            BPL => { cycles += self.branch(pc.wrapping_add(1), ! self.p.n()) },
            BMI => { cycles += self.branch(pc.wrapping_add(1), self.p.n()) },
            BNE => { cycles += self.branch(pc.wrapping_add(1), ! self.p.z()) },
            BEQ => { cycles += self.branch(pc.wrapping_add(1), self.p.z()) },
            BCC => { cycles += self.branch(pc.wrapping_add(1), ! self.p.c()) },
            BCS => { cycles += self.branch(pc.wrapping_add(1), self.p.c()) },
            BVC => { cycles += self.branch(pc.wrapping_add(1), ! self.p.v()) },
            BVS => { cycles += self.branch(pc.wrapping_add(1), self.p.v()) },
            BRK => self.handle_interrupt(true, IRQ_VECTOR_H, IRQ_VECTOR_L),
            CMP_IMM => {
                let value = self.memory.get(pc.wrapping_add(1));
                self.cmp(self.a, value)
            },
            CMP_ZP | CMP_ZP_X | CMP_ABS | CMP_ABS_X | CMP_ABS_Y | CMP_IND_X | CMP_IND_Y | CMP_ZPI_65C02 => {
                if opcode == CMP_ZPI_65C02 && ! self.is_65c02 {}
                else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.cmp(self.a, value);
                    cycles += self.run_indirect(opcode, pc, address, CMP_IND_Y, CMP_ABS_X, CMP_ABS_Y);
                }
            },
            CPX_IMM => {
                let value = self.memory.get(pc.wrapping_add(1));
                self.cmp(self.x, value)
            },
            CPX_ZP | CPX_ABS => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                self.cmp(self.x, value);
            },
            CPY_IMM => {
                let value = self.memory.get(pc.wrapping_add(1));
                self.cmp(self.y, value)
            },
            CPY_ZP | CPY_ABS => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                self.cmp(self.y, value);
            },
            DEC_ZP | DEC_ZP_X | DEC_ABS | DEC_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                if opcode != DEC_ABS && opcode != DEC_ZP { resolved_address = Some(address) }
                resolved_address = Some(address);
                resolved_value = Some(value);
                self.dec_op(pc, addressing_type);
            },
            EOR_IMM => {
                self.a = self.a ^ self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.a);
            },
            EOR_ZP | EOR_ZP_X | EOR_ABS | EOR_ABS_X | EOR_ABS_Y | EOR_IND_Y | EOR_IND_X | EOR_ZPI_65C02 => {
                if opcode == EOR_ZPI_65C02 && ! self.is_65c02 {} else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.a = self.a ^ value;
                    self.p.set_nz_flags(self.a);
                    cycles += self.run_indirect(opcode, pc, address, EOR_IND_Y, EOR_ABS_X, EOR_ABS_Y);
                }
            },
            CLC => self.p.set_c(false),
            SEC => self.p.set_c(true),
            CLI => self.p.set_i(false),
            SEI => self.p.set_i(true),
            CLD => self.p.set_d(false),
            SED => self.p.set_d(true),
            CLV => self.p.set_v(false),
            INC_ZP | INC_ZP_X | INC_ABS | INC_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                // Phantom read
                let _ = self.memory.get(address);
                let new_value = value.wrapping_add(1);
                self.memory.set(address, new_value);
                self.last_write = Some((address, new_value));
                self.p.set_nz_flags(new_value);
            },
            JMP => self.pc = self.memory.word(pc.wrapping_add(1)),
            JMP_IND => {
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                self.pc = address;
            },
            JMP_IND_ABS_X if self.is_65c02 => {
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                self.pc = address;
            }
            JSR => {
                // Fix test "20 55 13": Interleave the JSR and byte reads
                let return_address = pc.wrapping_add(2);
                self.push_byte(((return_address & 0xff00) >> 8) as u8);
                self.pc = self.memory.word(pc.wrapping_add(1));
                self.push_byte(return_address as u8);
            },
            LDX_IMM => {
                self.x = self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.x);
            },
            LDA_IMM => {
                self.a = self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.a);
            },
            LDA_ZP | LDA_ZP_X | LDA_ABS | LDA_ABS_X | LDA_ABS_Y | LDA_IND_X | LDA_IND_Y | LDA_ZPI_65C02 => {
                if (opcode == LDA_ZPI_65C02) && ! self.is_65c02 {}
                else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.a = value;
                    self.p.set_nz_flags(self.a);
                    cycles += self.run_indirect(opcode, pc, address, LDA_IND_Y, LDA_ABS_X, LDA_ABS_Y);
                }
            },
            LDX_IMM => {
                self.x = self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.x);
            },
            LDX_ZP | LDX_ZP_Y | LDX_ABS | LDX_ABS_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                self.x = value;
                self.p.set_nz_flags(self.x);
                cycles += self.run_indirect(opcode, pc, address, 0, 0, LDX_ABS_Y);
            },
            LDY_IMM => {
                self.y = self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.y);
            },
            LDY_ZP | LDY_ZP_X | LDY_ABS | LDY_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                self.y = value;
                self.p.set_nz_flags(self.y);
                cycles += self.run_indirect(opcode, pc, address, 0, LDY_ABS_X, 0);
            },
            LSR => self.a = self.lsr(self.a),
            LSR_ZP | LSR_ZP_X | LSR_ABS | LSR_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                let new_value = self.lsr(value);
                self.memory.set(address, new_value);
                self.last_write = Some((address, new_value));
            },
            ORA_IMM => {
                self.a = self.a | self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.a);
            },
            ORA_ZP | ORA_ZP_X | ORA_ABS | ORA_ABS_X | ORA_ABS_Y | ORA_IND_X | ORA_IND_Y | ORA_ZPI_65C02 => {
                if opcode == ORA_ZPI_65C02 && ! self.is_65c02 {} else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    self.a = self.a | value;
                    self.p.set_nz_flags(self.a);
                    cycles += self.run_indirect(opcode, pc, address, ORA_IND_Y, ORA_ABS_X, ORA_ABS_Y);
                }
            },
            TAX => {
                self.x = self.a;
                self.p.set_nz_flags(self.x);
            },
            TXA => {
                self.a = self.x;
                self.p.set_nz_flags(self.a);
            },
            DEX => {
                self.x = self.x.wrapping_sub(1);
                self.p.set_nz_flags(self.x);
            },
            INX => {
                self.x = self.x.wrapping_add(1);
                self.p.set_nz_flags(self.x);
            },
            INC_65C02 if self.is_65c02 => {
                self.a = self.a.wrapping_add(1);
                self.p.set_nz_flags(self.a);
            }
            DEC_65C02 if self.is_65c02 => {
                self.a = self.a.wrapping_sub(1);
                self.p.set_nz_flags(self.a);
            }
            TAY => {
                self.y = self.a;
                self.p.set_nz_flags(self.y);
            },
            TYA => {
                self.a = self.y;
                self.p.set_nz_flags(self.a);
            },
            DEY => {
                self.y = self.y.wrapping_sub(1);
                self.p.set_nz_flags(self.y);
            },
            INY => {
                self.y = self.y.wrapping_add(1);
                self.p.set_nz_flags(self.y);
            },
            ROL => {
                self.a = self.rol(self.a);
            },
            ROL_ZP | ROL_ZP_X | ROL_ABS | ROL_ABS_X => {
                self.rol_op(addressing_type, pc);
            },
            ROR => {
                self.a = self.ror(self.a);
            },
            ROR_ZP | ROR_ZP_X | ROR_ABS | ROR_ABS_X => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                let new_value = self.ror(value);
                self.memory.set(address, new_value);
                self.last_write = Some((address, new_value));
            },
            RTI => {
                let b = self.pop_byte();
                self.p.set_value(b);
                self.pc = self.pop_word();
            },
            RTS => {
                self.pc = self.pop_word().wrapping_add(1);
            },
            SBC_IMM => {
                let value = self.memory.get(pc.wrapping_add(1));
                self.sbc(value);
            },
            SBC_ZP |  SBC_ZP_X | SBC_ABS | SBC_ABS_X | SBC_ABS_Y | SBC_IND_X | SBC_IND_Y | SBC_ZPI_65C02 => {
                if opcode == SBC_ZPI_65C02 && ! self.is_65c02 {} else {
                    let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    resolved_value = Some(value);
                    resolved_read = false;
                    self.sbc(value);
                    cycles += self.run_indirect(opcode, pc, address, SBC_IND_Y, SBC_ABS_X, SBC_ABS_Y);
                }
            },
            STA_ZP | STA_ZP_X | STA_ABS | STA_ABS_X | STA_ABS_Y | STA_IND_X | STA_IND_Y | STA_ZPI_65C02 => {
                if opcode == STA_ZPI_65C02 && ! self.is_65c02 {} else {
                    let address = addressing_type.address(pc, self);
                    // let (address, value) = self.address_value(pc, addressing_type);
                    resolved_address = Some(address);
                    // resolved_value = Some(value);
                    resolved_read = false;
                    self.memory.set(address, self.a);
                    self.last_write = Some((address, self.a));
                    // No +1 cycle in case of page crossing for STA
                }
            },
            TXS => self.s = self.x,
            TSX => {
                self.x = self.s;
                self.p.set_nz_flags(self.x);
            },
            PHA => self.push_byte(self.a),
            PLA => {
                self.a = self.pop_byte();
                self.p.set_nz_flags(self.a);
            },
            PHX_65C02 if self.is_65c02 => self.push_byte(self.x),
            PHY_65C02 if self.is_65c02 => self.push_byte(self.y),
            PLX_65C02 if self.is_65c02 => {
                self.x = self.pop_byte();
                self.p.set_nz_flags(self.x);
            },
            PLY_65C02 if self.is_65c02 => {
                self.y = self.pop_byte();
                self.p.set_nz_flags(self.y);
            },
            PHP => {
                self.p.set_b(true);
                // self.p.set_reserved(true);
                self.push_byte(self.p.value());
            },
            PLP => {
                let b = self.pop_byte();
                self.p.set_value(b);
            },
            STX_ZP | STX_ZP_Y | STX_ABS => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                self.memory.set(address, self.x);
                self.last_write = Some((address, self.x));
            },
            STY_ZP | STY_ZP_X | STY_ABS => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                self.memory.set(address, self.y);
                self.last_write = Some((address, self.y));
            },
            TRB_ABS_65C02 | TRB_ZP_65C02 if self.is_65c02 => {
                (resolved_address, resolved_value) = self.tsb_or_rsb(pc, addressing_type, false);
                resolved_read = false;
            }
            TSB_ABS_65C02 | TSB_ZP_65C02 if self.is_65c02 => {
                (resolved_address, resolved_value) = self.tsb_or_rsb(pc, addressing_type, true);
                resolved_read = false;
            }
            STZ_ZP_65C02 | STZ_ZP_X_65C02 | STZ_ABS_65C02 | STZ_ABS_X_65C02 if self.is_65c02 => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                resolved_value = Some(value);
                resolved_read = false;
                self.memory.set(address, 0);
            }

            _ => {
                if self.is_65c02 {
                    let (ra, rv, c) = self.run_65c02_opcodes(operand, pc);
                    resolved_address = ra;
                    resolved_value = rv;
                    cycles += c;
                } else {
                    cycles += self.undocumented_6502_opcodes(operand, pc);
                }
            }
        }

        let mut debug = false;

        if let Some(range) = &config.trace_range {
            debug = (range.0..range.1).contains(&pc);
        } else {
            debug = false;
        }

        self.trace_cycles_in_progress =
            config.trace_cycles_start != 0 && config.trace_cycles_start <= self.cycles;
        debug |= self.trace_cycles_in_progress;

        if ! self.trace_in_progress {
            if let Some(pc_start) = config.trace_pc_start {
                // Turn on that trace if we're around that PC
                self.trace_in_progress =
                    (pc_start.wrapping_sub(10)..pc_start.wrapping_add(10)).contains(&pc);
            } else if let Some(pc_end) = config.trace_pc_stop {
                if self.trace_in_progress &&
                    (pc_end.wrapping_sub(10)..pc_end.wrapping_add(10)).contains(&pc) {
                    self.trace_in_progress = false;
                }
            } else {
                self.trace_in_progress = false;
            }
        }

        debug |= self.trace_in_progress;
        // if self.trace_in_progress { println!("TRACE IN PROGRESS"); }

        if let Some(pc_stop) = config.trace_pc_stop {
            self.end_pc_reached = true;
            debug = false;
        }

        // let count_line_ok = if let Some(count) = config.trace_count {
        //     if self.debug_line_count == 0 {
        //         debug = false;
        //     }
        //     self.debug_line_count > 0
        // } else {
        //     true
        // };

        debug |= config.debug_asm;

        ///
        /// Tracing the asm log
        ///
        if debug && self.pc < 0xc000 {
        // if self.asm_always || self.trace_in_progress || self.trace_cycles_in_progress ||
        //     (pc < 0xf000 && config.debug_asm && count_line_ok
        //     && ! self.end_pc_reached && self.in_range)
        // { // && self.pc < 0xf000 {
            if self.debug_line_count > 0 {
                self.debug_line_count -= 1;
            }
            if true {
                // Asynchronous logging
                if let Some(sender) = &self.logging_sender {
                    let byte1 = self.memory.get(pc.wrapping_add(1));
                    let byte2 = self.memory.get(pc.wrapping_add(2));
                    sender.send(Log(LogMsg::new(self.cycles, pc, operand.clone(), byte1, byte2,
                        resolved_address, resolved_value, resolved_read,
                        self.a, self.x, self.y, self.p.value(), self.s)));
                }
            } else {
                // let disassembly_line = self.memory.disassemble(&self.operands, pc);
                let disassembly_line = Disassemble::disassemble2(&self.operands, pc,
                    &operand, self.memory.get(pc.wrapping_add(1)), self.memory.get(pc.wrapping_add(2)));
                let d = RunDisassemblyLine::new(self.cycles, disassembly_line,
                    resolved_address, resolved_value, resolved_read, cycles,
                    self.a, self.x, self.y, self.p.value(), self.s);
                let stack = self.format_stack();
                // println!("{} {} {}", d.to_asm(), self.p, stack);
                // println!("{}", d.to_csv());
                if false {
                    if config.trace_to_file {
                        if config.csv {
                            log_file(d.to_csv())
                        } else {
                            log_file(format!("{} {} {}", d.to_asm(), self.p, stack));
                        }
                    }
                } else {
                    if config.trace_to_file && config.csv {
                        log::info!("{}", d.to_csv());
                    } else {
                        log::info!("{} {} {}", d.to_asm(), self.p, stack);
                    }
                }
            }
        }
        return cycles;
    }

    fn tsb_or_rsb(&mut self, pc: u16, addressing_type: AddressingType, is_tsb: bool) -> (Option<u16>, Option<u8>) {
        let (address, value) = self.address_value(pc, addressing_type);
        self.p.set_z((self.a & value) == 0);
        let new_value = if is_tsb {
            self.a | value
        } else {
            (self.a ^ 0xff) & value
        };
        // println!("A:{:02X} operand:{:02X} new_value:{:02X}", self.a, value, new_value);
        self.memory.set(address, new_value);
        (Some(address), Some(value))
    }

    fn run_65c02_opcodes(&mut self, operand: Operand, pc: u16) -> (Option<u16>, Option<u8>, u8) {
        let opcode = operand.opcode;
        let mut addressing_type = operand.addressing_type;
        let mut cycles = 0;

        let mut resolved_address: Option<u16> = None;
        let mut resolved_value: Option<u8> = None;

        let mut bb = |bit_number: u8, test_is_set: bool| {
            let (address, value) = self.address_value(pc, addressing_type);
            resolved_value = Some(value);
            let bit_value = 1 << bit_number;
            let branch = (test_is_set && (value & bit_value) != 0) || (! test_is_set && (value & bit_value) == 0);
            self.branch(pc.wrapping_add(2), branch);
        };

        match opcode {
            BBR_0_65C02 => { bb(0, false); }
            BBS_0_65C02 => { bb(0, true); }
            BBR_1_65C02 => { bb(1, false); }
            BBS_1_65C02 => { bb(1, true); }
            BBR_2_65C02 => { bb(2, false); }
            BBS_2_65C02 => { bb(2, true); }
            BBR_3_65C02 => { bb(3, false); }
            BBS_3_65C02 => { bb(3, true); }
            BBR_4_65C02 => { bb(4, false); }
            BBS_4_65C02 => { bb(4, true); }
            BBR_5_65C02 => { bb(5, false); }
            BBS_5_65C02 => { bb(5, true); }
            BBR_6_65C02 => { bb(6, false); }
            BBS_6_65C02 => { bb(6, true); }
            BBR_7_65C02 => { bb(7, false); }
            BBS_7_65C02 => { bb(7, true); }
            RMB0_ZP_65C02 => { self.rmb(pc, addressing_type, 0, false); }
            RMB1_ZP_65C02 => { self.rmb(pc, addressing_type, 1, false); }
            RMB2_ZP_65C02 => { self.rmb(pc, addressing_type, 2, false); }
            RMB3_ZP_65C02 => { self.rmb(pc, addressing_type, 3, false); }
            RMB4_ZP_65C02 => { self.rmb(pc, addressing_type, 4, false); }
            RMB5_ZP_65C02 => { self.rmb(pc, addressing_type, 5, false); }
            RMB6_ZP_65C02 => { self.rmb(pc, addressing_type, 6, false); }
            RMB7_ZP_65C02 => { self.rmb(pc, addressing_type, 7, false); }
            SMB0_ZP_65C02 => { self.rmb(pc, addressing_type, 0, true); }
            SMB1_ZP_65C02 => { self.rmb(pc, addressing_type, 1, true); }
            SMB2_ZP_65C02 => { self.rmb(pc, addressing_type, 2, true); }
            SMB3_ZP_65C02 => { self.rmb(pc, addressing_type, 3, true); }
            SMB4_ZP_65C02 => { self.rmb(pc, addressing_type, 4, true); }
            SMB5_ZP_65C02 => { self.rmb(pc, addressing_type, 5, true); }
            SMB6_ZP_65C02 => { self.rmb(pc, addressing_type, 6, true); }
            SMB7_ZP_65C02 => { self.rmb(pc, addressing_type, 7, true); }
            _ => {}
        };

        (resolved_address, resolved_value, cycles)
    }

    fn rmb(&mut self, pc: u16, addressing_type: AddressingType, bit_number: u8, set: bool) {
        let (address, value) = self.address_value(pc, addressing_type);
        let bit_value = 1 << bit_number;
        let new_value = if set { value | bit_value } else { value & (bit_value ^ 0xff) };
        self.memory.set(address, new_value);
    }

    fn undocumented_6502_opcodes(&mut self, operand: Operand, pc: u16) -> u8 {
        let opcode = operand.opcode;
        let mut addressing_type = operand.addressing_type;
        let mut cycles = operand.cycles;

        let mut resolved_address: Option<u16> = None;
        let mut resolved_value: Option<u8> = None;

        match opcode {
            NOP => { cycles = 0; }

            //
            // Non documented 6502 opcodes
            //
            NOP_8 | NOP_10 | NOP_11
            | NOP_ZP | NOP_13 | NOP_17 | NOP_19 | NOP_20
            => {}

            NOP_5C_ABS_X | NOP_7C_ABS_X | NOP_DC_ABS_X | NOP_FC_ABS_X
            => {
                let (address, _) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                let old = self.memory.word(pc.wrapping_add(1));
                cycles += self.page_crossed(old, address);
            }
            SLO_ZP | SLO_ZP_X | SLO_ABS | SLO_ABS_X | SLO_ABS_Y | SLO_IND_X | SLO_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                self.p.set_c((value & 0x80) != 0);
                let value = (value << 1);
                self.memory.set(address, value);
                self.a = self.a | value;
                self.p.set_nz_flags(self.a);
            }
            LAX_ZP | LAX_ZP_Y | LAX_ABS | LAX_ABS_Y | LAX_IND_X | LAX_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                self.a = value;
                self.x = value;
                self.p.set_nz_flags(value);
                let old = self.memory.word(pc.wrapping_add(1));
                cycles += self.page_crossed(old, address);
            }
            ISC_ZP | ISC_ZP_X | ISC_ABS | ISC_ABS_X | ISC_ABS_Y | ISC_IND_X | ISC_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                let new_value = value.wrapping_add(1);
                self.memory.set(address, new_value);
                self.sbc(new_value);
            }
            DCP_ZP | DCP_ZP_X | DCP_ABS | DCP_ABS_X | DCP_ABS_Y | DCP_IND_X | DCP_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                self.dec_op(pc, addressing_type);
                self.cmp(self.a, value.wrapping_sub(1));
            }
            RLA_ZP | RLA_ZP_X | RLA_ABS | RLA_ABS_X | RLA_ABS_Y | RLA_IND_X | RLA_IND_Y => {
                self.rol_op(addressing_type, pc);
                let address = self.and_op(addressing_type, pc);
                resolved_address = Some(address);
                cycles += self.run_indirect(opcode, pc, address, AND_IND_Y, AND_ABS_X, AND_ABS_Y);
            }
            RRA_ZP | RRA_ZP_X | RRA_ABS | RRA_ABS_X | RRA_ABS_Y | RRA_IND_X | RRA_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                let new_value = self.ror(value);
                self.memory.set(address, new_value);
                self.adc(address);
            }
            SRE_ZP | SRE_ZP_X | SRE_ABS | SRE_ABS_X | SRE_ABS_Y | SRE_IND_X | SRE_IND_Y => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                let new_value = self.lsr(value);
                self.memory.set(address, new_value);
                self.a = self.a ^ new_value;
                self.p.set_nz_flags(self.a);
            }
            ALR_IMM => {
                self.a = self.a & self.memory.get(pc.wrapping_add(1));
                self.a = self.lsr(self.a);
            }
            ANC_IMM | ANC2_IMM => {
                self.a = self.a & self.memory.get(pc.wrapping_add(1));
                self.p.set_nz_flags(self.a);
                self.p.set_c((self.a & (1 << 7)) != 0);
            }
            ANE_IMM => {
                // buggy
                let value = self.memory.get(pc.wrapping_add(1));
                self.a = self.x & value;
                self.p.set_nz_flags(self.a);
            }
            ARR_IMM => {
                // buggy
                let value = self.memory.get(pc.wrapping_add(1));
                self.a = self.a & value;
            }
            LAS => {
                let (address, value) = self.address_value(pc, addressing_type);
                resolved_address = Some(address);
                let new_value = self.s & value;
                self.a = new_value;
                self.x = new_value;
                self.s = new_value;
                self.p.set_nz_flags(new_value);
            }
            SAX_ZP | SAX_ZP_Y | SAX_ABS | SAX_IND_X => {
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                let value = self.x & self.a;
                self.memory.set(address, value);
            }
            SHA_ABS_Y | SHA_IND_Y => {
                // buggy
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                let value = self.memory.get(pc.wrapping_add(2)).wrapping_add(1);
                // println!("3rd value: {:02X}", value);
                let new_value = self.x & self.a;// & value;
                // println!("Storing [{:04X}]={:02X}", address, new_value);
                self.memory.set(address, new_value);
            }
            TAS_ABS_Y => {
                // buggy
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                let v = self.a & self.x;
                self.s = v;
                let v2 = self.memory.get(self.pc.wrapping_sub(1)).wrapping_add(1);
                let v3 = v & v2;
                self.memory.set(address, v & v2);
            }
            SHY_ABS_X => {
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                let value = self.y & (self.memory.get(self.pc.wrapping_sub(1)).wrapping_add(1));
                self.memory.set(address, value);
            }
            SHX_ABS_X => {
                let address = addressing_type.address(pc, self);
                resolved_address = Some(address);
                let value = self.x & (self.memory.get(self.pc.wrapping_sub(1)).wrapping_add(1));
                self.memory.set(address, value);
            }
            LXA_IMM => {
                let value = self.memory.get(pc.wrapping_add(1));
                // println!("or: {:02X} {:02X}", self.a, value);
                self.a = (self.a & value) | 0xea;
                self.x = self.a;
                self.p.set_nz_flags(self.x);
            }
            SBX_IMM => {
                let old_a = self.a;
                self.a = self.x & self.a;
                self.p.set_c(true);
                let v = self.memory.get(pc.wrapping_add(1));
                self.sbc(v);
                self.x = self.a;
                self.a = old_a;
            }
            _ => {}
        }
        cycles
    }

    fn ror(&mut self, v: u8) -> u8 {
        let bit0 = v & 1;
        let result = (v >> 1) | (self.p.c() as u8) << 7;
        self.p.set_nz_flags(result);
        self.p.set_c(bit0 != 0);
        result
    }

    fn and_op(&mut self, addressing_type: AddressingType, pc: u16) -> u16 {
        let (address, value) = self.address_value(pc, addressing_type);
        self.a &= value;
        self.p.set_nz_flags(self.a);
        address
    }

    fn rol_op(&mut self, addressing_type: AddressingType, pc: u16) {
        let (address, value) = self.address_value(pc, addressing_type);
        let new_value = self.rol(value);
        self.memory.set(address, new_value);
        self.last_write = Some((address, new_value));
    }

    fn rol(&mut self, v: u8) -> u8 {
        let result = (v << 1) | self.p.c() as u8;
        self.p.set_c(v & 0x80 != 0);
        self.p.set_nz_flags(result);
        result
    }

    fn lsr(&mut self, v: u8) -> u8 {
        let bit0 = v & 1;
        self.p.set_c(bit0 != 0);
        let result = v >> 1;
        self.p.set_nz_flags(result);
        result
    }

    fn cmp(&mut self, register: u8, v: u8) {
        // let tmp: i8 = 0;
        let tmp: i8 = (register as i16 - v as i16) as i8;
        self.p.set_c(register >= v);
        self.p.set_z(tmp == 0);
        self.p.set_n(tmp < 0);
    }

    fn handle_interrupt(&mut self, brk: bool, vector_high: u16, vector_low: u16) {
        self.p.set_b(brk);
        // Klaus functional tests require to increment the PC by 1
        // but the exhaustive 6502 tests require 2
        self.push_word((self.pc.wrapping_add(1)));
        self.push_byte(self.p.value());
        if self.is_65c02 {
            // BRK clears the D flag on 65C02
            self.p.set_d(false);
        }
        self.p.set_i(true);
        let new_pc = (self.memory.get(vector_high) as u16) << 8 | self.memory.get(vector_low) as u16;
        self.pc = new_pc;
    }

    /// TODO:
    /// add 1 cycle if branch on same page
    /// add 2 cycles otherwise
    fn branch(&mut self, pc: u16, condition: bool) -> u8 {
        let byte = self.memory.get(pc);
        let mut result = 0;
        if condition {
            let old = self.pc;
            self.pc = self.pc.wrapping_add(byte as u16);
            if byte >= 0x80 {
                self.pc = (self.pc as i64 - 0x100) as u16;
            }
            result += 1 + self.page_crossed(old, self.pc);
        }
        self.pc = self.pc & 0xffff;
        result
    }

    fn page_crossed(&self, old: u16, new: u16) -> u8 {
        if ((old ^ new) & 0xff00) > 0 { 1 } else { 0 }
        // let result = if ((old ^ new) & 0xff00) > 0 { 1 } else { 0 };
        // // println!("Calculating page cross {:04X} {:04X}: {}", old, new, result);
        // result
    }

    fn asl(&mut self, v: u8) -> u8 {
        self.p.set_c(v & 0x80 != 0);
        let result: u8 = v << 1;
        self.p.set_nz_flags(result);
        return result;
    }

    fn adc(&mut self, address: u16) {
        let v = self.memory.get(address);
        // println!("ADC A={:02X} V={:02X}", self.a, v);
        if self.p.d() {
            // println!("A is {:02X}, value:{:02X}, N flag: {}", self.a, v, (self.a & 0x80) != 0);
            let c = self.p.c() as u8;

            // 2a. AL = (A & $0F) + (B & $0F) + C
            // 2b. If AL >= $0A, then AL = ((AL + $06) & $0F) + $10
            // 2c. A = (A & $F0) + (B & $F0) + AL, using signed (twos complement) arithmetic
            // 2e. The N flag result is 1 if bit 7 of A is 1, and is 0 if bit 7 if A is 0
            // 2f. The V flag result is 1 if A < -128 or A > 127, and is 0 if -128 <= A <= 127
            let mut al = (self.a & 0xf) + (v & 0xf) + c;
            if al >= 10 {
                al = ((al.wrapping_add(6)) & 0xf) + 0x10;
            }
            let mut ah = (self.a >> 4) + (v >> 4) + if al > 0xf {1} else {0};

            if self.is_65c02 {
                self.p.set_n(false);
            } else {
                self.p.set_n(ah & 0x08 != 0);
            }
            self.p.set_v(!(self.a ^ v) & (self.a ^ (ah << 4)) &0x80 != 0);

            if ah >= 10 {
                ah = ah.wrapping_add(6);
            }

            self.p.set_c(ah > 15);
            self.p.set_z(self.a.wrapping_add(v).wrapping_add(c) == 0);

            // let new_a = (self.a & 0xf0) as i8 + (v & 0xf0) as i8 + al as i8;
            let new_a = (ah << 4) | (al & 0xf);
            self.a = new_a & 0xff;

            if self.is_65c02 {
                self.p.set_n(ah & 0x08 != 0);
            }

            } else {
            self.add(v);
        }
    }

    fn add(&mut self, v: u8) {
        let result: u16 = self.a as u16 + v as u16 + self.p.c() as u16;
        // NOTE: Parentheses are important here! Remove them and carry6 is incorrectly calculated
        let carry6 = (self.a & 0x7f) + (v & 0x7f) + self.p.c() as u8;
        self.p.set_c(result & 0x100 != 0);
        self.p.set_v(self.p.c() ^ (carry6 & 0x80 != 0));
        let result2 = result as u8;
        self.p.set_nz_flags(result2);
        self.a = result2;
    }


    fn sbc(&mut self, v: u8) {
        // println!("SBC A={:02X} V={:02X}", self.a, v);

        if self.p.d() {
            let c = if self.p.c() as u8 == 0 { 1 } else { 0 };
            let diff: u16 = (self.a as u16).wrapping_sub(v as u16).wrapping_sub(c as u16);
            let mut al: u8 = (self.a & 0x0f).wrapping_sub(v & 0x0f).wrapping_sub(c);
            if al &0x80 != 0 {
                al = al.wrapping_sub(6);
            }
            let mut ah: u8 = (self.a >> 4).wrapping_sub(v >> 4)
                .wrapping_sub(if al & 0x80 != 0 {1} else {0});
            self.p.set_z((diff & 0xff) == 0);
            self.p.set_n(diff & 0x80 != 0);
            self.p.set_v((self.a ^ v) & (self.a ^ ((diff & 0xff) as u8)) & 0x80 != 0);
            self.p.set_c(diff & 0xff00 == 0);
            if ah & 0x80 != 0 {
                ah = ah.wrapping_sub(6);
            }
            self.a = (ah << 4) | (al & 0x0f);
        } else {
            self.add((v ^ 0xff));
        }
    }

    fn inc(&mut self) {
        self.s = self.s.wrapping_add(1);
    }

    fn dec_op(&mut self, pc: u16, addressing_type: AddressingType) {
        let (address, value) = self.address_value(pc, addressing_type);
        let new_value = value.wrapping_sub(1);
        self.memory.set(address, new_value);
        self.last_write = Some((address, new_value));
        self.p.set_nz_flags(new_value);
    }

    fn dec(&mut self) {
        self.s = self.s.wrapping_sub(1);
    }

    pub(crate) fn push_byte(&mut self, a: u8) {
        self.memory.set(STACK_ADDRESS.wrapping_add(self.s as u16), a);
        self.dec();
    }

    pub(crate) fn pop_byte(&mut self,) -> u8 {
        self.inc();
        self.memory.get(STACK_ADDRESS.wrapping_add(self.s as u16))
    }

    pub(crate) fn push_word(&mut self, a: u16) {
        self.memory.set(STACK_ADDRESS.wrapping_add(self.s as u16), ((a & 0xff00) >> 8) as u8);
        self.dec();
        self.memory.set(STACK_ADDRESS.wrapping_add(self.s as u16), (a & 0xff) as u8);
        self.dec();
    }

    pub(crate) fn pop_word(&mut self) -> u16 {
        self.inc();
        let low = self.memory.get(STACK_ADDRESS.wrapping_add(self.s as u16)) as u16;
        self.inc();
        let high = self.memory.get(STACK_ADDRESS.wrapping_add(self.s as u16)) as u16;
        low | high << 8
    }

    /// Return a vector of the return addresses found on the stack
    pub(crate) fn stack_content(&mut self) -> Vec<u16> {
        let mut result = Vec::new();
        let down = std::cmp::max(self.s.wrapping_add(1), 0xf8);
        let mut i = 0xff_u8;
        if self.s < 0xff {
            loop {
                let v0 = self.memory.get(STACK_ADDRESS.wrapping_add(i as u16)) as u16;
                let v1 = self.memory.get(STACK_ADDRESS.wrapping_add(i as u16 - 1)) as u16;
                result.push((v0 << 8) | v1 +1);
                i = i - 2;
                if i < down { break; }
            }
        }
        result
    }

    pub(crate) fn format_stack(&mut self) -> String {
        let mut result = Vec::new();
        result.push(std::format!("SP={{${:2X} stack:[", self.s));
        for address in self.stack_content() {
            result.push(std::format!("{:04X} ", address));
        }

        result.push("]".to_string());
        result.join(" ")
    }

}
