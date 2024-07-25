use std::fmt::{Display, Formatter};
use crate::cpu::StatusFlags;
use crate::operand::Operand;

/// Representation of static disassembly
#[derive(Debug, Clone)]
pub struct DisassemblyLine {
    pub pc: u16,
    pub op: Operand,
    /// This vector will have the size of op.size: 0, 1, or 2
    pub bytes: Vec<u8>,
    /// e.g. "LDA"
    pub name: String,
    /// e.g. "$(6502,X)"
    pub value: String,
    /// Fully formatted line, e.g. "A0 02 65    LDA ($6502)"
    line: String,
    pub operand_size: u8,
}

impl Display for DisassemblyLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_asm()).unwrap();
        Ok(())
    }
}

impl DisassemblyLine {
    pub fn to_asm(&self) -> String {
        format!("{:<30}", self.line)
    }
}

/// Representation of runtime disassembly
pub struct RunDisassemblyLine {
    pub total_cycles: u128,
    pub disassembly_line: DisassemblyLine,
    pub resolved_address: Option<u16>,
    pub resolved_value: Option<u8>,
    pub resolved_read: bool,
    pub cycles: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
}

impl Display for RunDisassemblyLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_asm()).unwrap();
        Ok(())
    }
}

impl RunDisassemblyLine {
    pub fn new(total_cycles: u128, disassembly_line: DisassemblyLine,
        resolved_address: Option<u16>, resolved_value: Option<u8>, resolved_read: bool, cycles: u8,
        a: u8, x: u8, y: u8, p: u8, s: u8)
    -> Self {
        Self {
            total_cycles, disassembly_line, resolved_address, resolved_value, resolved_read, cycles,
                a, x, y, p, s
        }
    }

    pub fn to_csv(&self) -> String {
        let mut result: Vec<String> = Vec::new();
        result.push(format!("{:08}", self.total_cycles));
        result.push(format!("{:04X}", self.disassembly_line.pc));
        result.push(self.disassembly_line.line.to_string());
        result.push(if let Some(r) = self.resolved_address { format!("{:02X}", r) } else { "".to_string() });
        result.push(if let Some(r) = self.resolved_value { format!("{:02X}", r) } else { "".to_string() });
        result.push(format!("A={:02X}", self.a));
        result.push(format!("X={:02X}", self.x));
        result.push(format!("Y={:02X}", self.y));
        result.push(format!("P={:02X}", self.p));
        result.push(format!("S={:02X}", self.s));

        result.join("_")
    }

    fn format_resolved(&self) -> String {
        let rr = if self.resolved_read { "a" } else { "A" };
        let s = match (self.resolved_address, self.resolved_value){
            (Some(a), Some(v)) => {
                format!("{rr}${:04X}:v${:02X}", a, v)
            }
            (Some(a), None) => {
                format!("{rr}${:04X}", a)
            }
            (None, Some(v)) => {
                format!("v${:02X}", v)
            }
            (None, None) => {
                "".to_string()
            }
        };
        format!("{s:>12}")
    }

    /// Format:
    /// 24F550F4 9B 00 FF 01F8 N.RB.I.C  B7B0:C9 9B     CMP #$9B
    pub fn to_log(&self) -> String { 
        let line = self.disassembly_line.to_asm();
        let flags = StatusFlags::new_with(self.p);
        let result = format!("{:08} {:02X} {:02X} {:02X} {:04X} {flags} {:<10} | {}",
            self.total_cycles, self.a, self.x, self.y, 0x100 + self.s as u16, line,
            self.format_resolved()
        );
        result
    }

    pub fn to_asm(&self) -> String {
        let mut result = String::new();
        let ra = self.format_resolved();
        let line = self.disassembly_line.to_asm();
        let registers = std::format!("A={:02X} X={:02X} Y={:02X} P={:02X} S={:02X} PC={:04X}",
            self.a, self.x, self.y, self.p, self.s, self.disassembly_line.pc);
        result.push_str(&format!("{:08}| {:<30} {} {}",
            self.total_cycles,
            line,
            ra,
            registers));

        result
    }
}

pub struct Disassemble;

impl Disassemble {
    pub fn disassemble_range(memory: &[u8], operands: &[Operand], address: usize, stop: usize) -> Vec<DisassemblyLine> {
        let mut result: Vec<DisassemblyLine> = Vec::new();
        let mut current = address as u16;
        while current < stop as u16 {
            let dl = Disassemble::disassemble(memory, operands, current);
            current += dl.operand_size as u16;
            result.push(dl);
        }
        result
    }

    pub fn disassemble(memory: &[u8], operands: &[Operand], address: u16) -> DisassemblyLine {
        let opcode = memory[address as usize];
        let op = &operands[opcode as usize];
        let byte1 = memory[address.wrapping_add(1) as usize];
        let byte2 = memory[address.wrapping_add(2) as usize];
        Disassemble::disassemble2(operands, address, op, byte1, byte2)
    }
    
    pub fn disassemble2(operands: &[Operand], address: u16,
        op: &Operand, byte1: u8, byte2: u8)
            -> DisassemblyLine {
        let bytes: Vec<u8> =
            if op.size == 1 {
                Vec::new()
            } else if op.size == 2 {
                vec![byte1]
            } else {
                vec![byte1, byte2]
            };

        let line = Operand::disassemble_bytes(address, operands, op.opcode, byte1, byte2);

        DisassemblyLine {
            op: *op,
            pc: address,
            name: op.name.to_string(),
            bytes,
            value: op.addressing_type.to_string(address, byte1, (byte2 as u16) << 8 | byte1 as u16),
            line,
            operand_size: op.size,
        }
    }

    pub fn disassemble_multiple(memory: &Vec<u8>, operands: &[Operand], mut index: u16,
        line_count: u16)
            -> Vec<DisassemblyLine> {
        let mut result: Vec<DisassemblyLine> = Vec::new();
        for _ in 0..line_count {
            let l = Disassemble::disassemble(memory, operands, index);
            index = index.wrapping_add(l.operand_size as u16);
            result.push(l);
        }
        result
    }

}
