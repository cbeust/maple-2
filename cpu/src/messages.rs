use crate::operand::Operand;

pub struct LogMsg {
    pub global_cycles: u128,
    pub pc: u16,
    pub operand: Operand,
    pub byte1: u8,
    pub byte2: u8,
    pub resolved_address: Option<u16>,
    pub resolved_value: Option<u8>,
    pub resolved_read: bool,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub s: u8,
}

impl LogMsg {
    pub(crate) fn new(global_cycles: u128, pc: u16, operand: Operand, byte1: u8, byte2: u8,
        resolved_address: Option<u16>, resolved_value: Option<u8>, resolved_read: bool,
        a: u8, x: u8, y: u8, p: u8, s: u8) -> Self {
        Self {
            global_cycles, pc, operand, byte1, byte2, resolved_address, resolved_value,
            resolved_read, a, x, y, p, s
        }
    }
}

pub enum ToLogging {
    Log(LogMsg),
    End,
    Exit,
}

pub enum ToCpuUi {
    LogStarted,
    LogEnded,
}
