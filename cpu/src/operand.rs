use crate::addressing_type::AddressingType;
use crate::constants::{OPERANDS_6502, OPERANDS_65C02};

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    pub opcode: u8,
    pub size: u8,  // 1, 2, or 3 bytes
    pub(crate) name: &'static str,
    pub(crate) cycles: u8,
    pub(crate) addressing_type: AddressingType,
}

impl Operand {
    /// Returns the formatter string for this opcode with the (optional) given operands.
    /// * `pc`: The current PC, displayed on the left side
    /// * `byte1` and `byte2`: Possible data to the operands
    pub(crate) fn disassemble(&self, pc: u16, b1: u8, b2: u8) -> String {
        let result = match self.size {
            3 => {
                format!("{:04X}: {:02X} {:02X} {:02X}   {} {}", pc,
                        self.opcode, b1, b2,
                        self.name,
                        self.addressing_type.to_string(pc, b1, b1 as u16 | (b2 as u16) << 8))
            },
            2 => {
                format!("{:04X}: {:02X} {:02X}      {} {}", pc,
                        self.opcode, b1,
                        self.name,
                        self.addressing_type.to_string(pc, b1, 0))
            },
            _ => {
                format!("{:04X}: {:02X}         {}", pc,
                        self.opcode,
                        self.name)
            }
        }.to_string();
        result
    }

    pub(crate) fn disassemble_bytes(pc: u16, operands: &[Operand], operand: u8, b1: u8, b2: u8) -> String {
        operands[operand as usize].disassemble(pc, b1, b2)
    }
}

