#![allow(unused)]
#![allow(warnings)]

pub const DEFAULT_EMULATOR_SPEED_HZ: u64 = 1_300_000;

pub(crate) const IRQ_VECTOR_L: u16 = 0xfffe;
pub(crate) const IRQ_VECTOR_H: u16 = 0xffff;

pub const BRK: u8 = 0x00;
pub const ORA_IND_X: u8 = 0x01;
pub const JAM_02: u8 = 0x02;
pub const NOP_02_65C02: u8 = 0x02;
pub const SLO_IND_X: u8 = 0x03;
pub const NOP_03_65C02: u8 = 0x02;
pub const NOP_ZP: u8 = 0x04;
pub const TSB_ZP_65C02: u8 = 0x04;
pub const ORA_ZP: u8 = 0x05;
pub const ASL_ZP: u8 = 0x06;
pub const SLO_ZP: u8 = 0x07;
pub const RMB0_ZP_65C02: u8 = 0x07;
pub const PHP: u8 = 0x08;
pub const ORA_IMM: u8 = 0x09;
pub const ASL: u8 = 0xa;
pub const ANC_IMM: u8 = 0x0b;
pub const NOP_0B_65C02: u8 = 0x0B;
pub const NOP_ABS: u8 = 0x0c;
pub const TSB_ABS_65C02: u8 = 0x0c;
pub const ORA_ABS: u8 = 0x0d;
pub const ASL_ABS: u8 = 0xe;
pub const BBR_0_65C02: u8 = 0xf;
pub const SLO_ABS: u8 = 0xf;

pub const BPL: u8 = 0x10;
pub const ORA_IND_Y: u8 = 0x11;
pub const JAM_12: u8 = 0x12;
pub const ORA_ZPI_65C02: u8 = 0x12;
pub const SLO_IND_Y: u8 = 0x13;
pub const NOP_13_65C02: u8 = 0x13;
pub const NOP_ZP_X: u8 = 0x14;
pub const TRB_ZP_65C02: u8 = 0x14;
pub const ORA_ZP_X: u8 = 0x15;
pub const ASL_ZP_X: u8 = 0x16;
pub const SLO_ZP_X: u8 = 0x17;
pub const RMB1_ZP_65C02: u8 = 0x17;
pub const CLC: u8 = 0x18;
pub const ORA_ABS_Y: u8 = 0x19;
pub const NOP_1: u8 = 0x1a;
pub const INC_65C02: u8 = 0x1a;
pub const SLO_ABS_Y: u8 = 0x1b;
pub const NOP_1B_65C02: u8 = 0x1b;
pub const TRB_ABS_65C02: u8 = 0x1c;
pub const ORA_ABS_X: u8 = 0x1d;
pub const ASL_ABS_X: u8 = 0x1e;
pub const BBR_1_65C02: u8 = 0x1f;
pub const SLO_ABS_X: u8 = 0x1f;

pub const JSR: u8 = 0x20;
pub const AND_IND_X: u8 = 0x21;
pub const JAM_22: u8 = 0x22;
pub const NOP_22_65C02: u8 = 0x22;
pub const NOP_23_65C02: u8 = 0x23;
pub const RLA_IND_X: u8 = 0x23;
pub const BIT_ZP: u8 = 0x24;
pub const AND_ZP: u8 = 0x25;
pub const ROL_ZP: u8 = 0x26;
pub const RLA_ZP: u8 = 0x27;
pub const RMB2_ZP_65C02: u8 = 0x27;
pub const PLP: u8 = 0x28;
pub const AND_IMM: u8 = 0x29;
pub const ROL: u8 = 0x2a;
pub const ANC2_IMM: u8 = 0x2b;
pub const NOP_2B_65C02: u8 = 0x2b;
pub const BIT_ABS: u8 = 0x2c;
pub const AND_ABS: u8 = 0x2d;
pub const ROL_ABS: u8 = 0x2e;
pub const BBR_2_65C02: u8 = 0x2f;
pub const RLA_ABS: u8 = 0x2f;

pub const BMI: u8 = 0x30;
pub const AND_IND_Y: u8 = 0x31;
pub const JAM_3: u8 = 0x32;
pub const AND_ZPI_65C02: u8 = 0x32;
pub const RLA_IND_Y: u8 = 0x33;
pub const NOP_33_65C02: u8 = 0x33;
pub const NOP_16: u8 = 0x34;
pub const BIT_ZP_X_65C02: u8 = 0x34;
pub const AND_ZP_X: u8 = 0x35;
pub const ROL_ZP_X: u8 = 0x36;
pub const RMB3_ZP_65C02: u8 = 0x37;
pub const RLA_ZP_X: u8 = 0x37;
pub const SEC: u8 = 0x38;
pub const AND_ABS_Y: u8 = 0x39;
pub const NOP_2: u8 = 0x3a;
pub const DEC_65C02: u8 = 0x3a;
pub const RLA_ABS_Y: u8 = 0x3b;
pub const NOP_3B_65C02: u8 = 0x3b;
pub const NOP_3C_ABS_X: u8 = 0x3c;
pub const BIT_ABS_X_65C02: u8 = 0x3c;
pub const AND_ABS_X: u8 = 0x3d;
pub const ROL_ABS_X: u8 = 0x3e;
pub const BBR_3_65C02: u8 = 0x3f;
pub const RLA_ABS_X: u8 = 0x3f;

pub const RTI: u8 = 0x40;
pub const EOR_IND_X: u8 = 0x41;
pub const JAM_4: u8 = 0x42;
pub const NOP_42_65C02: u8 = 0x42;
pub const SRE_IND_X: u8 = 0x43;
pub const NOP_43_65C02: u8 = 0x43;
pub const NOP_13: u8 = 0x44;
pub const NOP_44_65C02: u8 = 0x44;
pub const EOR_ZP: u8 = 0x45;
pub const LSR_ZP: u8 = 0x46;
pub const SRE_ZP: u8 = 0x47;
pub const RMB4_ZP_65C02: u8 = 0x47;
pub const PHA: u8 = 0x48;
pub const EOR_IMM: u8 = 0x49;
pub const LSR: u8 = 0x4a;
pub const ALR_IMM: u8 = 0x4b;
pub const NOP_4B_65C02: u8 = 0x4b;
pub const JMP: u8 = 0x4c;
pub const EOR_ABS: u8 = 0x4d;
pub const LSR_ABS: u8 = 0x4e;
pub const BBR_4_65C02: u8 = 0x4f;
pub const SRE_ABS: u8 = 0x4f;

pub const BVC: u8 = 0x50;
pub const EOR_IND_Y: u8 = 0x51;
pub const JAM_5: u8 = 0x52;
pub const EOR_ZPI_65C02: u8 = 0x52;
pub const SRE_IND_Y: u8 = 0x53;
pub const NOP_53_65C02: u8 = 0x53;
pub const NOP_17: u8 = 0x54;
pub const NOP_54_65C02: u8 = 0x54;
pub const EOR_ZP_X: u8 = 0x55;
pub const LSR_ZP_X: u8 = 0x56;
pub const SRE_ZP_X: u8 = 0x57;
pub const RMB5_ZP_65C02: u8 = 0x57;
pub const CLI: u8 = 0x58;
pub const EOR_ABS_Y: u8 = 0x59;
pub const NOP_3: u8 = 0x5a;
pub const PHY_65C02: u8 = 0x5a;
pub const SRE_ABS_Y: u8 = 0x5b;
pub const NOP_5B_65C02: u8 = 0x5b;
pub const NOP_5C_ABS_X: u8 = 0x5c;
pub const EOR_ABS_X: u8 = 0x5d;
pub const LSR_ABS_X: u8 = 0x5e;
pub const BBR_5_65C02: u8 = 0x5f;
pub const SRE_ABS_X: u8 = 0x5f;

pub const RTS: u8 = 0x60;
pub const ADC_IND_X: u8 = 0x61;
pub const JAM_6: u8 = 0x62;
pub const NOP_62_65C02: u8 = 0x62;
pub const RRA_IND_X: u8 = 0x63;
pub const NOP_63_65C02: u8 = 0x63;
pub const NOP_14: u8 = 0x64;
pub const STZ_ZP_65C02: u8 = 0x64;
pub const ADC_ZP: u8 = 0x65;
pub const ROR_ZP: u8 = 0x66;
pub const RRA_ZP: u8 = 0x67;
pub const RMB6_ZP_65C02: u8 = 0x67;
pub const PLA: u8 = 0x68;
pub const ADC_IMM: u8 = 0x69;
pub const ROR: u8 = 0x6a;
pub const ARR_IMM: u8 = 0x6b;
pub const NOP_6B_65C02: u8 = 0x6b;
pub const JMP_IND: u8 = 0x6c;
pub const ADC_ABS: u8 = 0x6d;
pub const ROR_ABS: u8 = 0x6e;
pub const BBR_6_65C02: u8 = 0x6f;
pub const RRA_ABS: u8 = 0x6f;

pub const BVS: u8 = 0x70;
pub const ADC_IND_Y: u8 = 0x71;
pub const JAM_7: u8 = 0x72;
pub const ADC_ZPI_65C02: u8 = 0x72;
pub const RRA_IND_Y: u8 = 0x73;
pub const NOP_73_65C02: u8 = 0x73;
pub const NOP_18: u8 = 0x74;
pub const STZ_ZP_X_65C02: u8 = 0x74;
pub const ADC_ZP_X: u8 = 0x75;
pub const ROR_ZP_X: u8 = 0x76;
pub const RRA_ZP_X: u8 = 0x77;
pub const RMB7_ZP_65C02: u8 = 0x77;
pub const SEI: u8 = 0x78;
pub const ADC_ABS_Y: u8 = 0x79;
pub const NOP_4: u8 = 0x7a;
pub const PLY_65C02: u8 = 0x7a;
pub const NOP_7A_ABS_X: u8 = 0x7a;
pub const RRA_ABS_Y: u8 = 0x7b;
pub const NOP_7B_65C02: u8 = 0x7b;
pub const NOP_7C_ABS_X: u8 = 0x7c;
pub const JMP_IND_ABS_X: u8 = 0x7c;
pub const ADC_ABS_X: u8 = 0x7d;
pub const ROR_ABS_X: u8 = 0x7e;
pub const BBR_7_65C02: u8 = 0x7f;
pub const RRA_ABS_X: u8 = 0x7f;

pub const NOP_7: u8 = 0x80;
pub const BRA_65C02: u8 = 0x80;
pub const STA_IND_X: u8 = 0x81;
pub const NOP_8: u8 = 0x82;
pub const NOP_82_65C02: u8 = 0x82;
pub const SAX_IND_X: u8 = 0x83;
pub const NOP_83_65C02: u8 = 0x83;
pub const STY_ZP: u8 = 0x84;
pub const STA_ZP: u8 = 0x85;
pub const STX_ZP: u8 = 0x86;
pub const SAX_ZP: u8 = 0x87;
pub const SMB0_ZP_65C02: u8 = 0x87;
pub const DEY: u8 = 0x88;
pub const NOP_9: u8 = 0x89;
pub const BIT_IMM_65C02: u8 = 0x89;
pub const TXA: u8 = 0x8a;
pub const ANE_IMM: u8 = 0x8b;
pub const NOP_8B_65C02: u8 = 0x8b;
pub const STY_ABS: u8 = 0x8c;
pub const STA_ABS: u8 = 0x8d;
pub const STX_ABS: u8 = 0x8e;
pub const BBS_0_65C02: u8 = 0x8f;
pub const SAX_ABS: u8 = 0x8f;

pub const BCC: u8 = 0x90;
pub const STA_IND_Y: u8 = 0x91;
pub const JAM_9: u8 = 0x92;
pub const STA_ZPI_65C02: u8 = 0x92;
pub const SHA_ABS_Y: u8 = 0x93;
pub const NOP_93_65C02: u8 = 0x93;
pub const STY_ZP_X: u8 = 0x94;
pub const STA_ZP_X: u8 = 0x95;
pub const STX_ZP_Y: u8 = 0x96;
pub const SAX_ZP_Y: u8 = 0x97;
pub const SMB1_ZP_65C02: u8 = 0x97;
pub const TYA: u8 = 0x98;
pub const STA_ABS_Y: u8 = 0x99;
pub const TXS: u8 = 0x9a;
pub const TAS_ABS_Y: u8 = 0x9b;
pub const NOP_9B_65C02: u8 = 0x9b;
pub const SHY_ABS_X: u8 = 0x9c;
pub const STZ_ABS_65C02: u8 = 0x9c;
pub const STA_ABS_X: u8 = 0x9d;
pub const SHX_ABS_X: u8 = 0x9e;
pub const STZ_ABS_X_65C02: u8 = 0x9e;
pub const BBS_1_65C02: u8 = 0x9f;
pub const SHA_IND_Y: u8 = 0x9f;

pub const LDY_IMM: u8 = 0xa0;
pub const LDA_IND_X: u8 = 0xa1;
pub const LDX_IMM: u8 = 0xa2;
pub const LAX_IND_X: u8 = 0xa3;
pub const NOP_A3_65C02: u8 = 0xa3;
pub const LDY_ZP: u8 = 0xa4;
pub const LDA_ZP: u8 = 0xa5;
pub const LDX_ZP: u8 = 0xa6;
pub const LAX_ZP: u8 = 0xa7;
pub const SMB2_ZP_65C02: u8 = 0xa7;
pub const TAY: u8 = 0xa8;
pub const LDA_IMM: u8 = 0xa9;
pub const TAX: u8 = 0xaa;
pub const LXA_IMM: u8 = 0xab;
pub const NOP_AB_65C02: u8 = 0xab;
pub const LDY_ABS: u8 = 0xac;
pub const LDA_ABS: u8 = 0xad;
pub const LDX_ABS: u8 = 0xae;
pub const BBS_2_65C02: u8 = 0xaf;
pub const LAX_ABS: u8 = 0xaf;

pub const BCS: u8 = 0xb0;
pub const LDA_IND_Y: u8 = 0xb1;
pub const JAM_B: u8 = 0xb2;
pub const LDA_ZPI_65C02: u8 = 0xb2;
pub const LAX_IND_Y: u8 = 0xb3;
pub const NOP_B3_65C02: u8 = 0xb3;
pub const LDY_ZP_X: u8 = 0xb4;
pub const LDA_ZP_X: u8 = 0xb5;
pub const LDX_ZP_Y: u8 = 0xb6;
pub const LAX_ZP_Y: u8 = 0xb7;
pub const SMB3_ZP_65C02: u8 = 0xb7;
pub const CLV: u8 = 0xb8;
pub const LDA_ABS_Y: u8 = 0xb9;
pub const TSX: u8 = 0xba;
pub const LAS: u8 = 0xbb;
pub const NOP_BB_65C02: u8 = 0xbb;
pub const LDY_ABS_X: u8 = 0xbc;
pub const LDA_ABS_X: u8 = 0xbd;
pub const LDX_ABS_Y: u8 = 0xbe;
pub const BBS_3_65C02: u8 = 0xbf;
pub const LAX_ABS_Y: u8 = 0xbf;

pub const CPY_IMM: u8 = 0xc0;
pub const CMP_IND_X: u8 = 0xc1;
pub const NOP_10: u8 = 0xc2;
pub const NOP_C2_65C02: u8 = 0xc2;
pub const DCP_IND_X: u8 = 0xc3;
pub const NOP_C3_65C02: u8 = 0xc3;
pub const CPY_ZP: u8 = 0xc4;
pub const CMP_ZP: u8 = 0xc5;
pub const DEC_ZP: u8 = 0xc6;
pub const DCP_ZP: u8 = 0xc7;
pub const SMB4_ZP_65C02: u8 = 0xc7;
pub const INY: u8 = 0xc8;
pub const CMP_IMM: u8 = 0xc9;
pub const NOP_6: u8 = 0xfa;
pub const DEX: u8 = 0xca;
pub const SBX_IMM: u8 = 0xcb;
pub const NOP_CB_65C02: u8 = 0xcb;
pub const CPY_ABS: u8 = 0xcc;
pub const CMP_ABS: u8 = 0xcd;
pub const DEC_ABS: u8 = 0xce;
pub const BBS_4_65C02: u8 = 0xcf;
pub const DCP_ABS: u8 = 0xcf;

pub const BNE: u8 = 0xd0;
pub const CMP_IND_Y: u8 = 0xd1;
pub const CMP_ZPI_65C02: u8 = 0xd2;
pub const DCP_IND_Y: u8 = 0xd3;
pub const NOP_D3_65C02: u8 = 0xd3;
pub const NOP_19: u8 = 0xd4;
pub const NOP_D4_65C02: u8 = 0xd4;
pub const CMP_ZP_X: u8 = 0xd5;
pub const DEC_ZP_X: u8 = 0xd6;
pub const DCP_ZP_X: u8 = 0xd7;
pub const SMB5_ZP_65C02: u8 = 0xd7;
pub const CLD: u8 = 0xd8;
pub const CMP_ABS_Y: u8 = 0xd9;
pub const PHX_65C02: u8 = 0xda;
pub const DCP_ABS_Y: u8 = 0xdb;
pub const STP_65C02: u8 = 0xdb;
pub const NOP_DC_ABS_X: u8 = 0xdc;
pub const CMP_ABS_X: u8 = 0xdd;
pub const DEC_ABS_X: u8 = 0xde;
pub const BBS_5_65C02: u8 = 0xdf;
pub const DCP_ABS_X: u8 = 0xdf;

pub const CPX_IMM: u8 = 0xe0;
pub const SBC_IND_X: u8 = 0xe1;
pub const NOP_11: u8 = 0xe2;
pub const NOP_E2_65C02: u8 = 0xe2;
pub const ISC_IND_X: u8 = 0xe3;
pub const NOP_E3_65C02: u8 = 0xe3;
pub const CPX_ZP: u8 = 0xe4;
pub const SBC_ZP: u8 = 0xe5;
pub const INC_ZP: u8 = 0xe6;
pub const ISC_ZP: u8 = 0xe7;
pub const SMB6_ZP_65C02: u8 = 0xe7;
pub const INX: u8 = 0xe8;
pub const SBC_IMM: u8 = 0xe9;
pub const NOP: u8 = 0xea;
pub const USBC_IMM: u8 = 0xeb;
pub const NOP_EB_65C02: u8 = 0xeb;
pub const CPX_ABS: u8 = 0xec;
pub const SBC_ABS: u8 = 0xed;
pub const INC_ABS: u8 = 0xee;
pub const BBS_6_65C02: u8 = 0xef;
pub const ISC_ABS: u8 = 0xef;

pub const BEQ: u8 = 0xf0;
pub const SBC_IND_Y: u8 = 0xf1;
pub const JAM_F: u8 = 0xf2;
pub const SBC_ZPI_65C02: u8 = 0xf2;
pub const ISC_IND_Y: u8 = 0xf3;
pub const NOP_F3_65C02: u8 = 0xf3;
pub const NOP_20: u8 = 0xf4;
pub const NOP_F4_65C02: u8 = 0xf4;
pub const SBC_ZP_X: u8 = 0xf5;
pub const INC_ZP_X: u8 = 0xf6;
pub const ISC_ZP_X: u8 = 0xf7;
pub const SMB7_ZP_65C02: u8 = 0xf7;
pub const SED: u8 = 0xf8;
pub const SBC_ABS_Y: u8 = 0xf9;
pub const PLX_65C02: u8 = 0xfa;
pub const ISC_ABS_Y: u8 = 0xfb;
pub const NOP_FB_65C02: u8 = 0xfb;
pub const NOP_FC_ABS_X: u8 = 0xfc;
pub const NOP_FC_65C02: u8 = 0xfc;
pub const SBC_ABS_X: u8 = 0xfd;
pub const INC_ABS_X: u8 = 0xfe;
pub const BBS_7_65C02: u8 = 0xff;
pub const ISC_ABS_X: u8 = 0xff;

use crate::operand::Operand;
use crate::addressing_type::AddressingType::*;
use crate::addressing_type::AddressingType;

/// All the operands. Operands spelled in lowercase are not officially documented
pub const OPERANDS_6502: [Operand; 256] = [
    // Opcode, size, name, cycles, addressing type

    // 00-0f
    op(BRK, 1, "BRK", 7, Unknown),
    op(ORA_IND_X, 2, "ORA", 6, Indirect_X),
    op(JAM_02, 0, "jam", 3, Unknown),
    op(SLO_IND_X, 2, "slo", 8, Indirect_X),
    op(NOP_ZP, 2, "nop", 3, Zp),
    op(ORA_ZP, 2, "ORA", 3, Zp),
    op(ASL_ZP, 2, "ASL", 5, Zp),
    op(SLO_ZP, 2, "slo", 5, Zp),
    op(PHP, 1, "PHP", 3, Unknown),
    op(ORA_IMM, 2, "ORA", 2, Immediate),
    op(ASL, 1, "ASL", 2, Register_A),
    op(ANC_IMM, 2, "anc", 2, Immediate),
    op(NOP_ABS, 3, "nop", 4, Absolute),
    op(ORA_ABS, 3, "ORA", 4, Absolute),
    op(ASL_ABS, 3, "ASL", 6, Absolute),
    op(SLO_ABS, 3, "slo", 6, Absolute),

    // 10-1f
    op(BPL, 2, "BPL", 2, Relative),
    op(ORA_IND_Y, 2, "ORA", 5, Indirect_Y),
    op(JAM_12, 0, "nop", 3, Zpi),
    op(SLO_IND_Y, 2, "slo", 8, Indirect_Y),
    op(NOP_ZP_X, 1, "nop", 3, Zp_X),
    op(ORA_ZP_X, 2, "ORA", 4, Zp_X),
    op(ASL_ZP_X, 2, "ASL", 6, Zp_X),
    op(SLO_ZP_X, 2, "slo", 6, Zp_X),
    op(CLC, 1, "CLC", 2, Unknown),
    op(ORA_ABS_Y, 3, "ORA", 4, Absolute_Y),
    op(NOP_1, 1, "nop", 2, Unknown),
    op(SLO_ABS_Y, 3, "slo", 7, Absolute_Y),
    op(NOP_ABS, 1, "nop", 4, Absolute),
    op(ORA_ABS_X, 3, "ORA", 4, Absolute_X),
    op(ASL_ABS_X, 3, "ASL", 7, Absolute_X),
    op(SLO_ABS_X, 3, "slo", 7, Absolute_X),

    // 20-2f
    op(JSR, 3, "JSR", 6, Absolute),
    op(AND_IND_X, 2, "AND", 6, Indirect_X),
    op(JAM_22, 0, "jam", 3, Unknown),
    op(RLA_IND_X, 2, "rla", 8, Indirect_X),
    op(BIT_ZP, 2, "BIT", 3, Zp),
    op(AND_ZP, 2, "AND", 3, Zp),
    op(ROL_ZP, 2, "ROL", 5, Zp),
    op(RLA_ZP, 2, "rla", 5, Zp),
    op(PLP, 1, "PLP", 4, Unknown),
    op(AND_IMM, 2, "AND", 2, Immediate),
    op(ROL, 1, "ROL", 2, Register_A),
    op(ANC2_IMM, 2, "anc", 2, Immediate),
    op(BIT_ABS, 3, "BIT", 4, Absolute),
    op(AND_ABS, 3, "AND", 4, Absolute),
    op(ROL_ABS, 3, "ROL", 6, Absolute),
    op(RLA_ABS, 3, "rla", 6, Absolute),

    // 30-3f
    op(BMI, 2, "BMI", 2, Relative), // 0
    op(AND_IND_Y, 2, "AND", 5, Indirect_Y), // 1
    op(JAM_3, 0, "jam", 3, Zpi),
    op(RLA_IND_Y, 2, "rla", 8, Indirect_Y),
    op(NOP_16, 2, "nop", 4, Zp_X),
    op(AND_ZP_X, 2, "AND", 4, Zp_X),
    op(ROL_ZP_X, 2, "ROL", 6, Zp_X),
    op(RLA_ZP_X, 2, "rla", 6, Zp_X),
    op(SEC, 1, "SEC", 2, Unknown),   // 8
    op(AND_ABS_Y, 3, "AND", 4, Absolute_Y), // 9
    op(NOP_2, 1, "nop", 2, Unknown),
    op(ANC2_IMM, 2, "anc", 2, Immediate),
    op(NOP_3C_ABS_X, 3, "nop", 4, Absolute_X),
    op(AND_ABS_X, 3, "AND", 4, Absolute_X),
    op(ROL_ABS_X, 3, "ROL", 7, Absolute_X),
    op(RLA_ABS_X, 3, "rla", 7, Absolute_X),

    // 40-4f
    op(RTI, 1, "RTI", 6, Unknown),
    op(EOR_IND_X, 2, "EOR", 6, Indirect_X),
    op(JAM_4, 0, "jam", 3, Unknown),
    op(SRE_IND_X, 2, "sre", 8, Indirect_X),
    op(NOP_13, 2, "nop", 3, Zp),
    op(EOR_ZP, 2, "EOR", 3, Zp),
    op(LSR_ZP, 2, "LSR", 5, Zp),
    op(SRE_ZP, 2, "sre", 5, Zp),
    op(PHA, 1, "PHA", 3, Unknown),
    op(EOR_IMM, 2, "EOR", 2, Immediate),
    op(LSR, 1, "LSR", 2, Register_A),
    op(ALR_IMM, 2, "alr", 2, Immediate),
    op(JMP, 3, "JMP", 3, Absolute),
    op(EOR_ABS, 3, "EOR", 4, Absolute),
    op(LSR_ABS, 3, "LSR", 6, Absolute),
    op(SRE_ABS, 3, "sre", 6, Absolute),

    // 50-5f
    op(BVC, 2, "BVC", 2, Relative),
    op(EOR_IND_Y, 2, "EOR", 5, Indirect_Y),
    op(JAM_5, 0, "JAM", 3, Zpi),
    op(SRE_IND_Y, 2, "sre", 8, Indirect_Y),
    op(NOP_17, 2, "nop", 4, Zp_X),
    op(EOR_ZP_X, 2, "EOR", 4, Zp_X),
    op(LSR_ZP_X, 2, "LSR", 6, Zp_X),
    op(SRE_ZP_X, 2, "sre", 6, Zp_X),
    op(CLI, 1, "CLI", 2, Unknown),
    op(EOR_ABS_Y, 3, "EOR", 4, Absolute_Y),
    op(NOP_3, 1, "nop", 2, Unknown),
    op(SRE_ABS_Y, 3, "sre", 7, Absolute_Y),
    op(NOP_5C_ABS_X, 3, "nop", 4, Absolute_X),
    op(EOR_ABS_X, 3, "EOR", 4, Absolute_X),
    op(LSR_ABS_X, 3, "LSR", 7, Absolute_X),
    op(SRE_ABS_X, 3, "sre", 7, Absolute_X),

    // 60-6f
    op(RTS, 1, "RTS", 6, Unknown),
    op(ADC_IND_X, 2, "ADC", 6, Indirect_X),
    op(JAM_6, 0, "JAM", 3, Unknown),
    op(RRA_IND_X, 2, "rra", 8, Indirect_X),
    op(NOP_14, 2, "nop", 3, Zp),
    op(ADC_ZP, 2, "ADC", 3, Zp),
    op(ROR_ZP, 2, "ROR", 5, Zp),
    op(RRA_ZP, 2, "rra", 5, Zp),
    op(PLA, 1, "PLA", 4, Unknown),
    op(ADC_IMM, 2, "ADC", 2, Immediate),
    op(ROR, 1, "ROR", 2, Register_A),
    op(ARR_IMM, 2, "arr", 2, Immediate),
    op(JMP_IND, 3, "JMP", 5, Indirect),
    op(ADC_ABS, 3, "ADC", 4, Absolute),
    op(ROR_ABS, 3, "ROR", 6, Absolute),
    op(RRA_ABS, 3, "rra", 6, Absolute),

    // 70-7f
    op(BVS, 2, "BVS", 2, Relative),
    op(ADC_IND_Y, 2, "ADC", 5, Indirect_Y),
    op(JAM_7, 0, "JAM", 3, Zpi),
    op(RRA_IND_Y, 2, "rra", 8, Indirect_Y),
    op(NOP_18, 2, "nop", 4, Zp_X),
    op(ADC_ZP_X, 2, "ADC", 4, Zp_X),
    op(ROR_ZP_X, 2, "ROR", 6, Zp_X),
    op(RRA_ZP_X, 1, "rra", 1, Zp),
    op(NOP_4, 1, "nop", 2, Unknown),
    op(ADC_ABS_Y, 3, "ADC", 4, Absolute_Y),
    op(NOP_7A_ABS_X, 1, "nop", 2, Unknown),
    op(RRA_ABS_Y, 3, "rra", 7, Absolute_Y),
    op(NOP_7C_ABS_X, 3, "nop", 4, Absolute_X),
    op(ADC_ABS_X, 3, "ADC", 4, Absolute_X),
    op(ROR_ABS_X, 3, "ROR", 7, Absolute_X),
    op(RRA_ABS_X, 3, "rra", 7, Absolute_X),

    // 80-8f
    op(NOP_7, 2, "nop", 2, Immediate),
    op(STA_IND_X, 2, "STA", 6, Indirect_X),
    op(NOP_8, 2, "nop", 2, Immediate),
    op(SAX_IND_X, 2, "sax", 6, Indirect_X),
    op(STY_ZP, 2, "STY", 3, Zp),
    op(STA_ZP, 2, "STA", 3, Zp),
    op(STX_ZP, 2, "STX", 3, Zp),
    op(SMB0_ZP_65C02, 2, "SMB0", 5, Zp),
    op(DEY, 1, "DEY", 2, Unknown),
    op(NOP_9, 2, "nop", 2, Immediate),
    op(TXA, 1, "TXA", 2, Unknown),
    op(ANE_IMM, 2, "ane", 2, Immediate),
    op(STY_ABS, 3, "STY", 4, Absolute),
    op(STA_ABS, 3, "STA", 4, Absolute),
    op(STX_ABS, 3, "STX", 4, Absolute),
    op(SAX_ABS, 3, "sax", 4, Absolute),

    // 90-9f
    op(BCC, 2, "BCC", 2, Relative),
    op(STA_IND_Y, 2, "STA", 6, Indirect_Y),
    op(JAM_9, 0, "JAM", 3, Zpi),
    op(SHA_ABS_Y, 3, "sha", 6, Absolute_Y),
    op(STY_ZP_X, 2, "STY", 4, Zp_X),
    op(STA_ZP_X, 2, "STA", 4, Zp_X),
    op(STX_ZP_Y, 2, "STX", 4, Zp_Y),
    op(SAX_ZP_Y, 1, "sax", 1, Zp),
    op(TYA, 1, "TYA", 2, Unknown),
    op(STA_ABS_Y, 3, "STA", 5, Absolute_Y),
    op(TXS, 1, "TXS", 2, Unknown),
    op(TAS_ABS_Y, 3, "tas", 5, Absolute_Y),
    op(SHY_ABS_X, 3, "shy", 5, Absolute_X),
    op(STA_ABS_X, 3, "STA", 5, Absolute_X),
    op(SHX_ABS_X, 3, "shx", 5, Absolute_X),
    op(SHA_IND_Y, 2, "sha", 5, Indirect_Y),

    // a0-af
    op(LDY_IMM, 2, "LDY", 2, Immediate),
    op(LDA_IND_X, 2, "LDA", 6, Indirect_X),
    op(LDX_IMM, 2, "LDX", 2, Immediate),
    op(LAX_IND_X, 2, "lax", 6, Indirect_X),
    op(LDY_ZP, 2, "LDY", 3, Zp),
    op(LDA_ZP, 2, "LDA", 3, Zp),
    op(LDX_ZP, 2, "LDX", 3, Zp),
    op(SMB2_ZP_65C02, 2, "SMB2", 5, Zp),
    op(TAY, 1, "TAY", 2, Unknown),
    op(LDA_IMM, 2, "LDA", 2, Immediate),
    op(TAX, 1, "TAX", 2, Unknown),
    op(LXA_IMM, 2, "lxa", 2, Immediate),
    op(LDY_ABS, 3, "LDY", 4, Absolute),
    op(LDA_ABS, 3, "LDA", 4, Absolute),
    op(LDX_ABS, 3, "LDX", 4, Absolute),
    op(LAX_ABS, 3, "lax", 4, Absolute),

    // b0-bf
    op(BCS, 2, "BCS", 2, Relative),
    op(LDA_IND_Y, 2, "LDA", 5, Indirect_Y),
    op(JAM_B, 0, "JAM", 3, Zpi),
    op(LAX_IND_Y, 2, "lax", 5, Indirect_Y),
    op(LDY_ZP_X, 2, "LDY", 4, Zp_X),
    op(LDA_ZP_X, 2, "LDA", 4, Zp_X),
    op(LDX_ZP_Y, 2, "LDX", 4, Zp_Y),
    op(LAX_ZP_Y, 1, "lax", 1, Zp),
    op(CLV, 1, "CLV", 2, Unknown),
    op(LDA_ABS_Y, 3, "LDA", 4, Absolute_Y),
    op(TSX, 1, "TSX", 2, Unknown),
    op(LAS, 3, "las", 4, Absolute_Y),
    op(LDY_ABS_X, 3, "LDY", 4, Absolute_X),
    op(LDA_ABS_X, 3, "LDA", 4, Absolute_X),
    op(LDX_ABS_Y, 3, "LDX", 4, Absolute_Y),
    op(LAX_ABS_Y, 3, "lax", 4, Absolute_Y),

    // c0-cf
    op(CPY_IMM, 2, "CPY", 2, Immediate),
    op(CMP_IND_X, 2, "CMP", 6, Indirect_X),
    op(NOP_10, 2, "nop", 2, Immediate),
    op(DCP_IND_X, 2, "DCP", 8, Indirect_X),
    op(CPY_ZP, 2, "CPY", 3, Zp),
    op(CMP_ZP, 2, "CMP", 3, Zp),
    op(DEC_ZP, 2, "DEC", 5, Zp),
    op(SMB4_ZP_65C02, 2, "SMB4", 5, Zp),
    op(INY, 1, "INY", 2, Unknown),
    op(CMP_IMM, 2, "CMP", 2, Immediate),
    op(DEX, 1, "DEX", 2, Unknown),
    op(SBX_IMM, 2, "sbx", 2, Immediate),
    op(CPY_ABS, 3, "CPY", 4, Absolute),
    op(CMP_ABS, 3, "CMP", 4, Absolute),
    op(DEC_ABS, 3, "DEC", 6, Absolute),
    op(LAX_ABS_Y, 3, "lax", 4, Absolute_Y),

    // d0-df
    op(BNE, 2, "BNE", 2, Relative),
    op(CMP_IND_Y, 2, "CMP", 5, Indirect_Y),
    op(JAM_B, 0, "JAM", 3, Zpi),
    op(DCP_IND_Y, 2, "DCP", 8, Indirect_Y),
    op(NOP_19, 2, "nop", 4, Zp_X),
    op(CMP_ZP_X, 2, "CMP", 4, Zp_X),
    op(DEC_ZP_X, 2, "DEC", 6, Zp_X),
    op(DCP_ZP_X, 1, "nop", 1, Zp),
    op(CLD, 1, "CLD", 2, Unknown),
    op(CMP_ABS_Y, 3, "CMP", 4, Absolute_Y),
    op(0xda, 1, "nop", 2, Unknown),
    op(DCP_ABS_Y, 3, "DCP", 7, Absolute_Y),
    op(NOP_DC_ABS_X, 3, "nop", 4, Absolute_X),
    op(CMP_ABS_X, 3, "CMP", 4, Absolute_X),
    op(DEC_ABS_X, 3, "DEC", 7, Absolute_X),
    op(DCP_ABS_X, 3, "DCP", 7, Absolute_X),

    // e0-ea
    op(CPX_IMM, 2, "CPX", 2, Immediate),
    op(SBC_IND_X, 2, "SBC", 6, Indirect_X),
    op(NOP_11, 2, "nop", 2, Immediate),
    op(ISC_IND_X, 2, "isc", 8, Indirect_X),
    op(CPX_ZP, 2, "CPX", 3, Zp),
    op(SBC_ZP, 2, "SBC", 3, Zp),
    op(INC_ZP, 2, "INC", 5, Zp),
    op(ISC_ZP, 1, "nop", 1, Zp),
    op(INX, 1, "INX", 2, Unknown),
    op(SBC_IMM, 2, "SBC", 2, Immediate),
    op(NOP, 1, "NOP", 2, Unknown),
    op(USBC_IMM, 2, "usbc", 2, Unknown),
    op(CPX_ABS, 3, "CPX", 4, Absolute),
    op(SBC_ABS, 3, "SBC", 4, Absolute),
    op(INC_ABS, 3, "INC", 6, Absolute),
    op(ISC_ABS, 3, "isc", 6, Absolute),

    // f0-ff
    op(BEQ, 2, "BEQ", 2, Relative),
    op(SBC_IND_Y, 2, "SBC", 5, Indirect_Y),
    op(JAM_F, 0, "JAM", 3, Zpi),
    op(ISC_IND_Y, 2, "isc", 8, Indirect_Y),
    op(NOP_20, 2, "nop", 4, Zp_X),
    op(SBC_ZP_X, 2, "SBC", 4, Zp_X),
    op(INC_ZP_X, 2, "INC", 6, Zp_X),
    op(ISC_ZP_X, 1, "nop", 1, Zp),
    op(SED, 1, "SED", 2, Unknown),
    op(SBC_ABS_Y, 3, "SBC", 4, Absolute_Y),
    op(NOP_6, 1, "nop", 2, Unknown),
    op(ISC_ABS_Y, 3, "isc", 7, Absolute_Y),
    op(NOP_FC_ABS_X, 3, "nop", 4, Absolute_X),
    op(SBC_ABS_X, 3, "SBC", 4, Absolute_X),
    op(INC_ABS_X, 3, "INC", 7, Absolute_X),
    op(ISC_ABS_X, 3, "isc", 7, Absolute_X),
];

pub const OPERANDS_65C02: [Operand; 256] = [
    // Opcode, size, name, cycles, addressing type

    // 00-0f
    op(BRK, 1, "BRK", 7, Unknown),
    op(ORA_IND_X, 2, "ORA", 6, Indirect_X),
    op(NOP_02_65C02, 2, "nop", 2, Unknown),
    op(NOP_03_65C02, 1, "nop", 1, Unknown),
    op(TSB_ZP_65C02, 2, "TSB", 5, Zp),
    op(ORA_ZP, 2, "ORA", 3, Zp),
    op(ASL_ZP, 2, "ASL", 5, Zp),
    op(RMB0_ZP_65C02, 2, "RMB0", 5, Zp),
    op(PHP, 1, "PHP", 3, Unknown),
    op(ORA_IMM, 2, "ORA", 2, Immediate),
    op(ASL, 1, "ASL", 2, Register_A),
    op(NOP_0B_65C02, 1, "nop", 1, Zp),
    op(TSB_ABS_65C02, 3, "TSB", 5, Absolute),
    op(ORA_ABS, 3, "ORA", 4, Absolute),
    op(ASL_ABS, 3, "ASL", 6, Absolute),
    op(BBR_0_65C02, 3, "BBR0", 5, Zp_Relative),

    // 10-1f
    op(BPL, 2, "BPL", 2, Relative),
    op(ORA_IND_Y, 2, "ORA", 5, Indirect_Y),
    op(ORA_ZPI_65C02, 2, "ORA", 5, Zpi),
    op(NOP_13_65C02, 1, "jam", 1, Zpi),
    op(TRB_ZP_65C02, 2, "TRB", 5, Zp),
    op(ORA_ZP_X, 2, "ORA", 4, Zp_X),
    op(ASL_ZP_X, 2, "ASL", 6, Zp_X),
    op(RMB1_ZP_65C02, 2, "RMB1", 5, Zp),
    op(CLC, 1, "CLC", 2, Unknown),
    op(ORA_ABS_Y, 3, "ORA", 4, Absolute_Y),
    op(DEC_65C02, 1, "DEC", 2, Register_A),
    op(NOP_1B_65C02, 1, "nop", 1, Zp),
    op(TRB_ABS_65C02, 3, "TRB", 6, Absolute),
    op(ORA_ABS_X, 3, "ORA", 4, Absolute_X),
    op(ASL_ABS_X, 3, "ASL", 6, Absolute_X),
    op(BBR_1_65C02, 3, "BBR1", 5, Zp_Relative),

    // 20-2f
    op(JSR, 3, "JSR", 6, Absolute),
    op(AND_IND_X, 2, "AND", 6, Indirect_X),
    op(NOP_22_65C02, 2, "jam", 2, Unknown),
    op(NOP_23_65C02, 1, "nop", 1, Unknown),
    op(BIT_ZP, 2, "BIT", 3, Zp),
    op(AND_ZP, 2, "AND", 3, Zp),
    op(ROL_ZP, 2, "ROL", 5, Zp),
    op(RMB2_ZP_65C02, 2, "RMB2", 5, Zp),
    op(PLP, 1, "PLP", 4, Unknown),
    op(AND_IMM, 2, "AND", 2, Immediate),
    op(ROL, 1, "ROL", 2, Register_A),
    op(NOP_2B_65C02, 1, "nop", 1, Zp),
    op(BIT_ABS, 3, "BIT", 4, Absolute),
    op(AND_ABS, 3, "AND", 4, Absolute),
    op(ROL_ABS, 3, "ROL", 6, Absolute),
    op(BBR_2_65C02, 3, "BBR2", 5, Zp_Relative),

    // 30-3f
    op(BMI, 2, "BMI", 2, Relative),
    op(AND_IND_Y, 2, "AND", 5, Indirect_Y),
    op(AND_ZPI_65C02, 2, "AND", 5, Zpi),
    op(NOP_33_65C02, 1, "nop", 1, Indirect_Y),
    op(BIT_ZP_X_65C02, 2, "BIT", 4, Zp_X),
    op(AND_ZP_X, 2, "AND", 4, Zp_X),
    op(ROL_ZP_X, 2, "ROL", 6, Zp_X),
    op(RMB3_ZP_65C02, 2, "RMB3", 5, Zp),
    op(SEC, 1, "SEC", 2, Unknown),
    op(AND_ABS_Y, 3, "AND", 4, Absolute_Y),
    op(INC_65C02, 1, "INC", 2, Register_A),
    op(NOP_3B_65C02, 1, "nop", 1, Zp),
    op(BIT_ABS_X_65C02, 3, "BIT", 5, Absolute_X),
    op(AND_ABS_X, 3, "AND", 4, Absolute_X),
    op(ROL_ABS_X, 3, "ROL", 6, Absolute_X),
    op(BBR_3_65C02, 3, "BBR3", 5, Zp_Relative),

    // 40-4f
    op(RTI, 1, "RTI", 6, Unknown),
    op(EOR_IND_X, 2, "EOR", 6, Indirect_X),
    op(NOP_42_65C02, 2, "nop", 2, Unknown),
    op(NOP_43_65C02, 1, "nop", 1, Zp),
    op(NOP_44_65C02, 2, "nop", 2, Zp),
    op(EOR_ZP, 2, "EOR", 3, Zp),
    op(LSR_ZP, 2, "LSR", 5, Zp),
    op(RMB4_ZP_65C02, 2, "RMB4", 5, Zp),
    op(PHA, 1, "PHA", 3, Unknown),
    op(EOR_IMM, 2, "EOR", 2, Immediate),
    op(LSR, 1, "LSR", 2, Register_A),
    op(NOP_4B_65C02, 1, "nop", 1, Zp),
    op(JMP, 3, "JMP", 3, Absolute),
    op(EOR_ABS, 3, "EOR", 4, Absolute),
    op(LSR_ABS, 3, "LSR", 6, Absolute),
    op(BBR_4_65C02, 3, "BBR4", 5, Zp_Relative),

    // 50-5f
    op(BVC, 2, "BVC", 2, Relative),
    op(EOR_IND_Y, 2, "EOR", 5, Indirect_Y),
    op(EOR_ZPI_65C02, 2, "EOR", 5, Zpi),
    op(NOP_53_65C02, 1, "nop", 1, Zp),
    op(NOP_54_65C02, 2, "nop", 4, Zp_X),
    op(EOR_ZP_X, 2, "EOR", 4, Zp_X),
    op(LSR_ZP_X, 2, "LSR", 6, Zp_X),
    op(RMB5_ZP_65C02, 2, "RMB5", 5, Zp),
    op(CLI, 1, "CLI", 2, Unknown),
    op(EOR_ABS_Y, 3, "EOR", 4, Absolute_Y),
    op(PHY_65C02, 1, "PHY", 3, Unknown),
    op(NOP_5B_65C02, 1, "nop", 1, Zp),
    op(NOP_5C_ABS_X, 3, "nop", 4, Absolute_X),
    op(EOR_ABS_X, 3, "EOR", 4, Absolute_X),
    op(LSR_ABS_X, 3, "LSR", 6, Absolute_X),
    op(BBR_5_65C02, 3, "BBR5", 5, Zp_Relative),

    // 60-6f
    op(RTS, 1, "RTS", 6, Unknown),
    op(ADC_IND_X, 2, "ADC", 6, Indirect_X),
    op(NOP_62_65C02, 2, "nop", 2, Unknown),
    op(NOP_63_65C02, 1, "nop", 1, Unknown),
    op(STZ_ZP_65C02, 2, "STZ", 3, Zp),
    op(ADC_ZP, 2, "ADC", 3, Zp),
    op(ROR_ZP, 2, "ROR", 5, Zp),
    op(RMB6_ZP_65C02, 2, "RMB6", 5, Zp),
    op(PLA, 1, "PLA", 4, Unknown),
    op(ADC_IMM, 2, "ADC", 2, Immediate),
    op(ROR, 1, "ROR", 2, Register_A),
    op(NOP_6B_65C02, 1, "nop", 1, Zp),
    op(JMP_IND, 3, "JMP", 5, Indirect),
    op(ADC_ABS, 3, "ADC", 4, Absolute),
    op(ROR_ABS, 3, "ROR", 6, Absolute),
    op(BBR_6_65C02, 3, "BBR6", 5, Zp_Relative),

    // 70-7f
    op(BVS, 2, "BVS", 2, Relative),
    op(ADC_IND_Y, 2, "ADC", 5, Indirect_Y),
    op(ADC_ZPI_65C02, 2, "ADC", 5, Zpi),
    op(NOP_73_65C02, 1, "nop", 1, Unknown),
    op(STZ_ZP_X_65C02, 2, "STZ", 4, Zp_X),
    op(ADC_ZP_X, 2, "ADC", 4, Zp_X),
    op(ROR_ZP_X, 2, "ROR", 6, Zp_X),
    op(RMB7_ZP_65C02, 2, "RMB7", 5, Zp),
    op(SEI, 1, "SEI", 2, Unknown),
    op(ADC_ABS_Y, 3, "ADC", 4, Absolute_Y),
    op(PLY_65C02, 1, "PLY", 4, Unknown),
    op(NOP_7B_65C02, 1, "nop", 1, Zp),
    op(JMP_IND_ABS_X, 3, "JMP", 4, Indirect_Abs_X),
    op(ADC_ABS_X, 3, "ADC", 4, Absolute_X),
    op(ROR_ABS_X, 3, "ROR", 6, Absolute_X),
    op(BBR_7_65C02, 3, "BBR7", 5, Zp_Relative),

    // 80-8f
    op(BRA_65C02, 2, "BRA", 3, Relative),
    op(STA_IND_X, 2, "STA", 6, Indirect_X),
    op(NOP_82_65C02, 2, "nop", 2, Immediate),
    op(NOP_83_65C02, 1, "nop", 1, Unknown),
    op(STY_ZP, 2, "STY", 3, Zp),
    op(STA_ZP, 2, "STA", 3, Zp),
    op(STX_ZP, 2, "STX", 3, Zp),
    op(SMB0_ZP_65C02, 2, "SMB0", 5, Zp),
    op(DEY, 1, "DEY", 2, Unknown),
    op(BIT_IMM_65C02, 2, "BIT", 2, Immediate),
    op(TXA, 1, "TXA", 2, Unknown),
    op(NOP_8B_65C02, 1, "nop", 1, Zp),
    op(STY_ABS, 3, "STY", 4, Absolute),
    op(STA_ABS, 3, "STA", 4, Absolute),
    op(STX_ABS, 3, "STX", 4, Absolute),
    op(BBS_0_65C02, 3, "BBS0", 5, Zp_Relative),

    // 90-9f
    op(BCC, 2, "BCC", 2, Relative),
    op(STA_IND_Y, 2, "STA", 6, Indirect_Y),
    op(STA_ZPI_65C02, 2, "STA", 5, Zpi),
    op(NOP_93_65C02, 1, "nop", 1, Unknown),
    op(STY_ZP_X, 2, "STY", 4, Zp_X),
    op(STA_ZP_X, 2, "STA", 4, Zp_X),
    op(STX_ZP_Y, 2, "STX", 4, Zp_Y),
    op(SMB1_ZP_65C02, 2, "SMB1", 5, Zp),
    op(TYA, 1, "TYA", 2, Unknown),
    op(STA_ABS_Y, 3, "STA", 5, Absolute_Y),
    op(TXS, 1, "TXS", 2, Unknown),
    op(NOP_9B_65C02, 1, "nop", 1, Zp),
    op(STZ_ABS_65C02, 3, "STZ", 4, Absolute),
    op(STA_ABS_X, 3, "STA", 5, Absolute_X),
    op(STZ_ABS_X_65C02, 3, "STZ", 5, Absolute_X),
    op(BBS_1_65C02, 3, "BBS1", 5, Zp_Relative),

    // a0-af
    op(LDY_IMM, 2, "LDY", 2, Immediate),
    op(LDA_IND_X, 2, "LDA", 6, Indirect_X),
    op(LDX_IMM, 2, "LDX", 2, Immediate),
    op(NOP_A3_65C02, 1, "nop", 1, Unknown),
    op(LDY_ZP, 2, "LDY", 3, Zp),
    op(LDA_ZP, 2, "LDA", 3, Zp),
    op(LDX_ZP, 2, "LDX", 3, Zp),
    op(LAX_ZP, 2, "lax", 3, Zp),
    op(TAY, 1, "TAY", 2, Unknown),
    op(LDA_IMM, 2, "LDA", 2, Immediate),
    op(TAX, 1, "TAX", 2, Unknown),
    op(NOP_AB_65C02, 1, "nop", 1, Zp),
    op(LDY_ABS, 3, "LDY", 4, Absolute),
    op(LDA_ABS, 3, "LDA", 4, Absolute),
    op(LDX_ABS, 3, "LDX", 4, Absolute),
    op(BBS_2_65C02, 3, "BBS2", 5, Zp_Relative),

    // b0-bf
    op(BCS, 2, "BCS", 2, Relative),
    op(LDA_IND_Y, 2, "LDA", 5, Indirect_Y),
    op(LDA_ZPI_65C02, 2, "LDA", 5, Zpi),
    op(NOP_B3_65C02, 1, "nop", 1, Unknown),
    op(LDY_ZP_X, 2, "LDY", 4, Zp_X),
    op(LDA_ZP_X, 2, "LDA", 4, Zp_X),
    op(LDX_ZP_Y, 2, "LDX", 4, Zp_Y),
    op(SMB3_ZP_65C02, 2, "SMB3", 5, Zp),
    op(CLV, 1, "CLV", 2, Unknown),
    op(LDA_ABS_Y, 3, "LDA", 4, Absolute_Y),
    op(TSX, 1, "TSX", 2, Unknown),
    op(NOP_BB_65C02, 1, "nop", 1, Zp),
    op(LDY_ABS_X, 3, "LDY", 4, Absolute_X),
    op(LDA_ABS_X, 3, "LDA", 4, Absolute_X),
    op(LDX_ABS_Y, 3, "LDX", 4, Absolute_Y),
    op(BBS_3_65C02, 3, "BBS3", 5, Zp_Relative),

    // c0-cf
    op(CPY_IMM, 2, "CPY", 2, Immediate),
    op(CMP_IND_X, 2, "CMP", 6, Indirect_X),
    op(NOP_C2_65C02, 2, "nop", 2, Immediate),
    op(NOP_C3_65C02, 1, "nop", 1, Unknown),
    op(CPY_ZP, 2, "CPY", 3, Zp),
    op(CMP_ZP, 2, "CMP", 3, Zp),
    op(DEC_ZP, 2, "CMP", 3, Zp),
    op(SMB4_ZP_65C02, 2, "SMB4", 5, Zp),
    op(INY, 1, "INY", 2, Unknown),
    op(CMP_IMM, 2, "CMP", 2, Immediate),
    op(DEX, 1, "DEX", 2, Unknown),
    op(NOP_CB_65C02, 1, "nop", 1, Zp),
    op(CPY_ABS, 3, "CPY", 4, Absolute),
    op(CMP_ABS, 3, "CMP", 4, Absolute),
    op(DEC_ABS, 3, "DEC", 6, Absolute),
    op(BBS_4_65C02, 3, "BBS4", 5, Zp_Relative),

    // d0-df
    op(BNE, 2, "BNE", 2, Relative),
    op(CMP_IND_Y, 2, "CMP", 5, Indirect_Y),
    op(CMP_ZPI_65C02, 2, "CMP", 5, Zpi),
    op(NOP_D3_65C02, 1, "nop", 1, Unknown),
    op(NOP_D4_65C02, 2, "nop", 4, Zp_X),
    op(CMP_ZP_X, 2, "CMP", 4, Zp_X),
    op(DEC_ZP_X, 2, "DEC", 6, Zp_X),
    op(SMB5_ZP_65C02, 2, "SMB5", 5, Zp),
    op(CLD, 1, "CLD", 2, Unknown),
    op(CMP_ABS_Y, 3, "CMP", 4, Absolute_Y),
    op(PHX_65C02, 1, "PHX", 3, Unknown),
    op(STP_65C02, 2, "STP", 1, Unknown),
    op(NOP_DC_ABS_X, 3, "nop", 4, Absolute_X),
    op(CMP_ABS_X, 3, "CMP", 4, Absolute_X),
    op(DEC_ABS_X, 3, "DEC", 7, Absolute_X),
    op(BBS_5_65C02, 3, "BBS5", 5, Zp_Relative),

    // e0-ea
    op(CPX_IMM, 2, "CPX", 2, Immediate),
    op(SBC_IND_X, 2, "SBC", 6, Indirect_X),
    op(NOP_E2_65C02, 2, "nop", 2, Immediate),
    op(NOP_E3_65C02, 1, "nop", 1, Unknown),
    op(CPX_ZP, 2, "CPX", 3, Zp),
    op(SBC_ZP, 2, "SBC", 3, Zp),
    op(INC_ZP, 2, "SBC", 3, Zp),
    op(SMB6_ZP_65C02, 2, "SMB6", 5, Zp),
    op(INX, 1, "INX", 2, Unknown),
    op(SBC_IMM, 2, "SBC", 2, Immediate),
    op(NOP, 1, "NOP", 2, Unknown),
    op(NOP_EB_65C02, 1, "nop", 1, Zp),
    op(CPX_ABS, 3, "CPX", 4, Absolute),
    op(SBC_ABS, 3, "SBC", 4, Absolute),
    op(INC_ABS, 3, "INC", 6, Absolute),
    op(BBS_6_65C02, 3, "BBS6", 5, Zp_Relative),

    // f0-ff
    op(BEQ, 2, "BEQ", 2, Relative),
    op(SBC_IND_Y, 2, "SBC", 5, Indirect_Y),
    op(SBC_ZPI_65C02, 2, "SBC", 5, Zpi),
    op(NOP_F3_65C02, 1, "nop", 1, Unknown),
    op(NOP_F4_65C02, 2, "nop", 4, Zp_X),
    op(SBC_ZP_X, 2, "SBC", 4, Zp_X),
    op(INC_ZP_X, 2, "INC", 6, Zp_X),
    op(SMB7_ZP_65C02, 2, "SMB7", 5, Zp),
    op(SED, 1, "SED", 2, Unknown),
    op(SBC_ABS_Y, 3, "SBC", 4, Absolute_Y),
    op(PLX_65C02, 1, "PLX", 4, Unknown),
    op(NOP_FB_65C02, 1, "nop", 1, Zp),
    op(NOP_FC_65C02, 3, "nop", 4, Absolute_X),
    op(SBC_ABS_X, 3, "SBC", 4, Absolute_X),
    op(INC_ABS_X, 3, "INC", 7, Absolute_X),
    op(BBS_7_65C02, 3, "BBS7", 5, Zp_Relative),
];

const fn op(opcode: u8, size: u8, name: &'static str, cycles: u8,
            addressing_type: AddressingType) -> Operand {
    Operand { opcode, size, name, cycles, addressing_type }
}

/// http://6502.org/tutorials/65c02opcodes.html
pub const OPCODES_65C02: [u8; 46] = [
    0x04, 0x0c,
    0x72, 0x32, 0x32, 0x52, 0x12, 0xf2, 0x92,
    0x7c,
    0x04, 0x0c,
    0xf, 0x1f, 0x2f, 0x3f, 0x4f, 0x5f, 0x6f, 0x7f, 0x8f, 0x9f, 0xaf, 0xbf, 0xcf, 0xdf, 0xef, 0xff,
    0x7, 0x17, 0x27, 0x37, 0x47, 0x57, 0x67, 0x77, 0x87, 0x97, 0xa7, 0xb7, 0xc7, 0xd7, 0xe7, 0xf7,
    0xdb,
    0xcb,
];

pub const _OPCODES_65C02_DONE: [u8; 18] = [
    0xb2,
    0x14, 0x1c,
    0x64, 0x74, 0x9c, 0x9e,
    0x80,
    0xd2,
    0x3a, 0x1a,
    0x89, 0x34, 0x3c,
    0xda, 0x5a, 0xfa, 0x7a,
];