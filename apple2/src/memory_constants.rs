#![allow(dead_code)]

pub const EIGHTY_STORE_OFF: u16 = 0xc000;
pub const EIGHTY_STORE_ON: u16 = 0xc001;
pub const EIGHTY_STORE_STATUS: u16 = 0xc018;
pub const READ_AUX_MEM_OFF: u16 = 0xc002;
pub const READ_AUX_MEM_ON: u16 = 0xc003;
pub const READ_AUX_MEM_STATUS: u16 = 0xc013;
pub const WRITE_AUX_MEM_OFF: u16 = 0xc004;
pub const WRITE_AUX_MEM_ON: u16 = 0xc005;
pub const WRITE_AUX_MEM_STATUS: u16 = 0xc014;
pub const INTERNAL_CX_OFF: u16 = 0xc006;
pub const INTERNAL_CX_ON: u16 = 0xc007;
pub const INTERNAL_CX_STATUS: u16 = 0xc015;
pub const ALT_ZP_OFF: u16 = 0xc008;
pub const ALT_ZP_ON: u16 = 0xc009;
pub const ALT_ZP_STATUS: u16 = 0xc016;
pub const SLOT_C3_OFF: u16 = 0xc00a;
pub const SLOT_C3_ON: u16 = 0xc00b;
pub const SLOT_C3_STATUS: u16 = 0xc017;
pub const EIGHTY_COLUMNS_OFF: u16 = 0xc00c;
pub const EIGHTY_COLUMNS_ON: u16 = 0xc00d;
pub const ALT_CHAR_OFF: u16 = 0xc00e;
pub const ALT_CHAR_ON: u16 = 0xc00f;
pub const ALT_CHAR_STATUS: u16 = 0xc01e;
pub const EIGHTY_COLUMNS_STATUS: u16 = 0xc01f;
pub const TEXT_OFF: u16 = 0xc050;
pub const TEXT_ON: u16 = 0xc051;
pub const TEXT_STATUS: u16 = 0xc01a;
pub const MIXED_OFF: u16 = 0xc052;
pub const MIXED_ON: u16 = 0xc053;
pub const MIXED_STATUS: u16 = 0xc01b;
pub const PAGE_2_OFF: u16 = 0xc054;
pub const PAGE_2_ON: u16 = 0xc055;
pub const PAGE_2_STATUS: u16 = 0xc01c;
pub const HIRES_OFF: u16 = 0xc056;
pub const HIRES_ON: u16 = 0xc057;
pub const HIRES_STATUS: u16 = 0xc01d;

/// W: Set annunciator-3 output to 0
/// if IOUDIS set, turn on double hires
pub const AN3_ON: u16 = 0xc05e;
/// W: Set annunciator-3 output to 1
/// if IOUDIS set, turn off double hires
pub const AN3_OFF: u16 = 0xc05f;
pub const AN3_STATUS: u16 = 0xc046;  // bit 5

/// W: Disable IOU (Enable double hires and disable $C058-5F)
/// R: bit 7 = IOUDIS status
pub const IOU_DIS_ON: u16 = 0xc07e;
/// W: enable IOU (enable $C058-5F)
/// R: status of double hires
pub const IOU_DIS_OFF: u16 = 0xc07f;


pub const INTERNAL_C8_ROM_ON: u16 = 0xc300;
pub const INTERNAL_C8_ROM_OFF: u16 = 0xcfff;