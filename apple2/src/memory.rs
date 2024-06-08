use std::fs;
use crossbeam::channel::Sender;
pub use cpu::memory::{Memory, DefaultMemory};
use crate::constants::PC;
use crate::disk::disk_controller::{DiskController};
use crate::disk::disk_info::DiskInfo;
use crate::debug::hex_dump_at;
use crate::memory_constants::*;
use crate::messages::ToUi;
use crate::messages::ToUi::RgbModeUpdate;
use crate::roms::{DISK2_ROM};
use crate::send_message;

const MAIN: usize = 0;
const AUX: usize = 1;

#[macro_export]
macro_rules! is_set {
    ($self:expr, $v:expr) => {
        ($self.memories[0][$v as usize] & 0x80) != 0
    }
}

#[macro_export]
macro_rules! set_soft_switch {
    ($self:expr, $v:expr) => {
        // println!("TURNING ON {:04X}", $v);
        $self.memories[0][$v as usize] = 0x80
    }
}

#[macro_export]
macro_rules! clear_soft_switch {
    ($self:expr, $v:expr) => {
        // println!("TURNING OFF {:04X}", $v);
        $self.memories[0][$v as usize] = 0
    }
}

#[derive(Debug, PartialEq)]
enum RomReadType {
    Rom, Bank1, Bank2,
}

#[derive(Debug, PartialEq)]
enum RomWriteType {
    Rom, Bank1, Bank2
}

pub struct HighRam {
    /// Two banks for D000-DFFF
    pub(crate) banks: [[u8; 0x1000]; 2],
    /// Only one high ram, E000-FFFF
    pub(crate) high_ram: [u8; 0x2000],
}

impl Default for HighRam {
    fn default() -> Self {
        Self {
            banks: [[0; 0x1000]; 2],
            high_ram: [0; 0x2000],
        }
    }
}

pub struct Apple2Memory {
    // memory2: Memory2,
    sender: Option<Sender<ToUi>>,
    pub(crate) memories: [[u8; DefaultMemory::MEMORY_SIZE as usize]; 2],

    /// Extra text page
    pub extra_text_memory: [u8; 0x400],

    /// Language card banks
    read_rom: RomReadType,
    write_rom: RomWriteType,
    bank_write_count: u8,

    /// Double hires graphics
    dhg_previous_address: u16,
    dhg_iou_disabled: bool,
    dhg_rgb_mode: u8,
    dhg_rgb_flags: u8,

    /// High ram, controlled by alt_zp, $D000-$FFFF
    /// If alt_zp is off, use the first element (motherboard high ram)
    /// If alt_zp is on, use the second element (auxiliary card high ram)
    pub high_ram: [HighRam; 2],

    /// There is no way to read this status, so we maintain it here
    slot_c8_status: bool,

    pub(crate) disk_controller: DiskController,
}

impl Apple2Memory {
    pub(crate) fn on_reboot(&mut self) {
        self.dhg_rgb_mode = 0;
        self.dhg_rgb_flags = 0;
    }
}

impl Apple2Memory {
    pub(crate) fn new(disk_infos: [Option<DiskInfo>; 2], sender: Option<Sender<ToUi>>) -> Self {
        Self {
            // memory2: Memory2::new(),
            sender: sender.clone(),
            memories: [[0; DefaultMemory::MEMORY_SIZE as usize]; 2],
            bank_write_count: 0,
            read_rom: RomReadType::Rom,
            write_rom: RomWriteType::Rom,
            extra_text_memory: [0; 0x400],
            high_ram: [HighRam::default(), HighRam::default()],
            slot_c8_status: false,
            // disk_controller: DiskController::new(6, "apple2/files/master.dsk"),
            disk_controller: DiskController::new_with_filename(6, &disk_infos, sender.clone()),
            dhg_previous_address: 0,
            dhg_iou_disabled: false,
            dhg_rgb_mode: 0,
            dhg_rgb_flags: 0,
        }
    }

    pub(crate) fn aux_memory(&self) -> Vec<u8> {
        self.memories[AUX].to_vec()
    }

    pub fn load_roms(&mut self) {
        let enhanced_rom = include_bytes!("../files/Apple2e_Enhanced.rom");
        self.load_bytes(enhanced_rom, 0xc000, 0, 0, true /* main mem */);
        // self.load_bytes(&APPLE_2E_ROM, 0xc000, 0, 0, true /* main mem */);

        // Disk2 at $C600 in slot (aux mem)
        self.load_bytes(&DISK2_ROM, 0xc600, 0 /* skip */, 0x100, false /* aux mem */);
    }

    fn log_mem(&self, address: u16, s: &str) {
        log::info!("  {} | {}", s,
            &format!("{:04X} read_rom:{:?}, write_rom:{:?}, count:{}, alt_zp:{}",
                address,
                self.read_rom, self.write_rom,
                self.bank_write_count, is_set!(self, ALT_ZP_STATUS)
            ))
    }

    /// Handle both get and set in the same function since we sometimes do the same thing
    /// for both accesses. Return `None` if we're setting a value, `Some(value)` if it's
    /// a memory get.
    fn get_or_set(&mut self, address: u16, value: u8, get: bool) -> Option<u8> {
        // if (0xc001..0xc080).contains(&address) {
        //     println!("Soft switch: {get} {:04X}", address);
        // }
        let mut result: Option<u8> = None;
        let set = ! get;

        // PRE-WRITE is set by odd read access in the $C08X range.
        // It is reset by even read access or any write access in the $C08X range.
        if (0xc080..=0xc08f).contains(&address) {
            let odd = (address & 1) == 1;
            if odd && get {
                if self.bank_write_count < 3 {
                    self.bank_write_count += 1;
                }
                #[cfg(feature = "log_memory")]
                self.log_mem(address, &format!("Incremented PREWRITE to {}", self.bank_write_count));
            } else if !odd || ! get {
                self.bank_write_count = 0;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Reset PREWRITE count to 0");
            }
        }

        // INTC8ROM: Unreadable soft switch (UTAIIe:5-28)
        // . Set:   On access to $C3XX with SLOTC3ROM reset
        //			- "From this point, $C800-$CFFF will stay assigned to motherboard ROM until
        //			   an access is made to $CFFF or until the MMU detects a system reset."
        // . Reset: On access to $CFFF or an MMU reset
        let slot = ((address & 0xff00) >> 8) & 0xf;
        if address == 0xcfff {
            // println!("CFFF ACCESSED, c8_status is now false");
            self.slot_c8_status = false;
        } else if (address & 0xc300) == 0xc300 && ! is_set!(self, SLOT_C3_STATUS) {
            // println!("c8_status is now true (accessed {:04X}", address);
            self.slot_c8_status = true;
        }

        match address {
            0..=0x1ff => {
                let index: usize = if is_set!(self, ALT_ZP_STATUS) { 1 } else { 0 };
                if get {
                    result = Some(self.memories[index][address as usize]);
                } else {
                    self.memories[index][address as usize] = value;
                    result = Some(0);
                }
            }
            0x200..=0xbfff => {
                let is_text = (0x400..=0x7ff).contains(&address);
                let is_graphics = (0x2000..=0x3fff).contains(&address);
                let is_hires_set = is_set!(&self, HIRES_STATUS);
                let is_eighty_set = is_set!(&self, EIGHTY_STORE_STATUS);
                let is_page2 = is_set!(&self, PAGE_2_STATUS);
                let is_read_aux = is_set!(&self, READ_AUX_MEM_STATUS);
                let is_write_aux = is_set!(&self, WRITE_AUX_MEM_STATUS);

                // Sather 5-25
                // If 80STORE is set, RAMRD and RAMWRT do not affect $400-$7ff
                // if 80STORE is set and HIRES are both set, RAMRD and RAMWRT do not affect $400-$7ff and $2000-$3fff
                // The PAGE2 and 80STORE inputs to Figure 5.3
                // are lOU soft switches. When set, PAGE2 selects secondary display memory pages for scanning.
                // 80STORE, when set, overrides the effect of PAGE2 on memory scanning, thus
                // inhibiting display of screen page 2.
                let index = if is_text {
                    if is_eighty_set {
                        if is_page2 { AUX } else { MAIN }
                    } else if (get && is_read_aux) || (!get && is_write_aux) { AUX } else { MAIN }
                } else if is_eighty_set && is_hires_set && (is_text || is_graphics) {
                    if is_page2 { AUX } else { MAIN }
                } else if (!get && is_write_aux) || (get && is_read_aux) { AUX } else { MAIN };

                if get {
                    result = Some(self.memories[index][address as usize]);
                } else {
                    self.memories[index][address as usize] = value;
                    result = Some(0);
                }
            }
            // Both R and W
            0xc080 | 0xc084 => {
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read bank 2, no write");
                self.read_rom = RomReadType::Bank2;
                self.write_rom = RomWriteType::Rom;
                result = Some(0)
            }
            // Both R and W
            0xc088 | 0xc08c => {
                self.read_rom = RomReadType::Bank1;
                self.write_rom = RomWriteType::Rom;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read bank 1, no write");
                result = Some(0)
            }

            // Both R and W
            0xc089 | 0xc08d => {
                self.read_rom = RomReadType::Rom;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read rom");
                if self.bank_write_count >= 2 {
                    self.write_rom = RomWriteType::Bank1;
                    #[cfg(feature = "log_memory")]
                    self.log_mem(address, "Write bank 2");
                }
                result = Some(0)
            }

            // Both R and W
            0xc081 | 0xc085 => {
                self.read_rom = RomReadType::Rom;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read rom");
                if self.bank_write_count >= 2 {
                    self.write_rom = RomWriteType::Bank2;
                    #[cfg(feature = "log_memory")]
                    self.log_mem(address, "Write bank 2");
                }
                result = Some(0)
            }
            // Both R and W
            0xc082 | 0xc086 | 0xc08a | 0xc08e => {
                self.read_rom = RomReadType::Rom;
                self.write_rom = RomWriteType::Rom;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read rom, no write");
                result = Some(0)
            }
            // Both R and W
            0xc083 | 0xc087 => {
                self.read_rom = RomReadType::Bank2;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read bank 2");
                if self.bank_write_count >= 2 {
                    self.write_rom = RomWriteType::Bank2;
                    #[cfg(feature = "log_memory")]
                    self.log_mem(address, "Write bank 2");
                }
                result = Some(0)
            }
            // Both R and W
            0xc08b | 0xc08f => {
                self.read_rom = RomReadType::Bank1;
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "Read bank 1");
                if self.bank_write_count >= 2 {
                    self.write_rom = RomWriteType::Bank1;
                    #[cfg(feature = "log_memory")]
                    self.log_mem(address, "Write bank 1");
                }
                result = Some(0)
            }

            EIGHTY_STORE_ON if set => { set_soft_switch!(self, EIGHTY_STORE_STATUS); }
            READ_AUX_MEM_OFF if set => { clear_soft_switch!(self, READ_AUX_MEM_STATUS); }
            READ_AUX_MEM_ON if set => { set_soft_switch!(self, READ_AUX_MEM_STATUS); }
            WRITE_AUX_MEM_OFF if set => { clear_soft_switch!(self, WRITE_AUX_MEM_STATUS); }
            WRITE_AUX_MEM_ON if set => { set_soft_switch!(self, WRITE_AUX_MEM_STATUS); }
            INTERNAL_CX_OFF if set => { clear_soft_switch!(self, INTERNAL_CX_STATUS); }
            INTERNAL_CX_ON if set => { set_soft_switch!(self, INTERNAL_CX_STATUS); }
            ALT_ZP_OFF if set => {
                clear_soft_switch!(self, ALT_ZP_STATUS);
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "ALT_ZERO_PAGE is off");
            }
            ALT_ZP_ON if set => {
                set_soft_switch!(self, ALT_ZP_STATUS);
                #[cfg(feature = "log_memory")]
                self.log_mem(address, "ALT_ZERO_PAGE is on");
            }
            SLOT_C3_OFF if set => { clear_soft_switch!(self, SLOT_C3_STATUS); }
            SLOT_C3_ON if set => { set_soft_switch!(self, SLOT_C3_STATUS); }
            EIGHTY_COLUMNS_OFF if set => { clear_soft_switch!(self, EIGHTY_COLUMNS_STATUS); }
            EIGHTY_COLUMNS_ON if set => { set_soft_switch!(self, EIGHTY_COLUMNS_STATUS); }
            ALT_CHAR_OFF if set => { clear_soft_switch!(self, ALT_CHAR_STATUS); }
            ALT_CHAR_ON if set => { set_soft_switch!(self, ALT_CHAR_STATUS); }
            // 0xc010 => if set { self.memories[MAIN][0xc000] &= 0x7f; }

            0xc100..=0xcffe => {
                // Sather, Understanding the Apple ][, 5-28
                let cx = is_set!(&self, INTERNAL_CX_STATUS);
                let c3 = is_set!(&self, SLOT_C3_STATUS);

                // Sather 5-28
                // internal is MAIN, slot is AUX
                //
                //  INTCXROM   SLOTC3ROM   $C100-$C2FF     $C300-$C3FF
                //                         $C400-$CFFF
                // ----------------------------------------------------
                //    false      false       slot            internal
                //    false      true        slot            slot
                //    true       false       internal        internal
                //    true       true        internal        internal
                let index = match(cx, c3) {
                    (false, false) => {
                        if slot >= 8 && self.slot_c8_status { MAIN }
                        else if slot == 3 { MAIN }
                        else { AUX }
                    }
                    (false, true) => {
                        if slot >= 8 && self.slot_c8_status {
                            MAIN
                        } else {
                            AUX
                        }
                    }
                    (true, false) => {
                        MAIN
                        // if slot == 3 || slot >= 8 {
                        //     if cx || self.slot_c8_status { MAIN } else { AUX }
                        // } else {
                        //     MAIN
                        // }
                    }
                    (true, true) => { MAIN }
                };

                let address = address as usize;
                let value = self.memories[index][address];
                if get {
                    result = Some(value);
                } else {
                    // self.memories[index][address] = value;
                    result = Some(0);
                }
            }
            0xd000..=0xffff => {
                let index: usize = if is_set!(self, ALT_ZP_STATUS) { 1 } else { 0 };
                // let (index, address) = self.memory_index_and_address(address);
                if get {
                    match self.read_rom {
                        RomReadType::Rom => {
                            result = Some(self.memories[0][address as usize]);
                            #[cfg(feature = "log_memory")]
                            self.log_mem(address, &format!("Read ROM {:04x}: {:02X}", address,
                                result.unwrap()));
                        }
                        RomReadType::Bank1 => {
                            if (0xd000..=0xdfff).contains(&address) {
                                result = Some(self.high_ram[index].banks[0][address as usize - 0xd000]);
                            } else {
                                result = Some(self.high_ram[index].high_ram[address as usize - 0xe000]);
                            }
                            #[cfg(feature = "log_memory")]
                            self.log_mem(address, &format!("Read Bank 1 {:04x}: {:02X}", address,
                                result.unwrap()));
                        }
                        RomReadType::Bank2 => {
                            if (0xd000..=0xdfff).contains(&address) {
                                result = Some(self.high_ram[index].banks[1][address as usize - 0xd000]);
                            } else {
                                result = Some(self.high_ram[index].high_ram[address as usize - 0xe000]);
                            }
                            #[cfg(feature = "log_memory")]
                            self.log_mem(address, &format!("Read Bank 2 {:04x}: {:02X}", address,
                                result.unwrap()));
                        }

                    }
                } else {
                    match self.write_rom {
                        RomWriteType::Rom => {
                            /* can't write to rom */
                            result = Some(0);
                        }
                        RomWriteType::Bank1 => {
                            if (0xd000..=0xdfff).contains(&address) {
                                self.high_ram[index].banks[0][address as usize - 0xd000] = value;
                            } else {
                                self.high_ram[index].high_ram[address as usize - 0xe000] = value;
                            }
                            result = Some(0);
                            #[cfg(feature = "log_memory")]
                            self.log_mem(address, &format!("Wrote Bank 1 {:04x}={:02X}", address,
                                value));
                        }
                        RomWriteType::Bank2 => {
                            if (0xd000..=0xdfff).contains(&address) {
                                self.high_ram[index].banks[1][address as usize - 0xd000] = value;
                            } else {
                                self.high_ram[index].high_ram[address as usize - 0xe000] = value;
                            }
                            result = Some(0);
                            #[cfg(feature = "log_memory")]
                            self.log_mem(address, &format!("Wrote Bank 2 {:04x}={:02X}", address,
                                value));
                        }
                    }
                }
            }
            0xc000 => {
                if get {
                    result = Some(self.memories[MAIN][address as usize]);
                } else {
                    clear_soft_switch!(self, EIGHTY_STORE_STATUS);
                }
            }
            0xc010 => {
                let value = self.memories[MAIN][0xc000] & 0x7f;
                if let Some(sender) = &self.sender {
                    sender.send(ToUi::KeyboardStrobe).unwrap();
                    self.memories[MAIN][0xc000] = value;
                    result = Some(value)
                }
            }
            0xc050 => { clear_soft_switch!(self, TEXT_STATUS); }
            0xc051 => { set_soft_switch!(self, TEXT_STATUS); }
            0xc052 => { clear_soft_switch!(self, MIXED_STATUS); }
            0xc053 => { set_soft_switch!(self, MIXED_STATUS); }
            0xc054 => { clear_soft_switch!(self, PAGE_2_STATUS); }
            0xc055 => {
                // If 80STORE On, PAGE_2_STATUS is now used to determine whether we write to aux mem
                // If 80STORE Off: Display Page 2
                set_soft_switch!(self, PAGE_2_STATUS);
            }
            0xc056 => { clear_soft_switch!(self, HIRES_STATUS); }
            0xc057 => { set_soft_switch!(self, HIRES_STATUS); }
            AN3_ON => {
                // Set AN3_STATUS
                self.memories[MAIN][AN3_STATUS as usize] |= 0b0010_0000;
                // Update the F1/F2 switches
                self.update_f1_f2(address);
                result = Some(0);
            }
            AN3_OFF => {
                // Clear AN3_STATUS
                self.memories[MAIN][AN3_STATUS as usize] &= 0b1101_1111;
                // Update the F1/F2 switches
                self.update_f1_f2(address);
                result = Some(0);
            }
            IOU_DIS_ON => {
                if get {
                    result = Some(if self.dhg_iou_disabled { 0x80 } else { 0 } );
                } else {
                    // Disable IOU
                    self.dhg_iou_disabled = true;
                }
            }
            IOU_DIS_OFF => {
                if get {
                    // status of double hires
                } else {
                    // Enable IOU
                    self.dhg_iou_disabled = false;
                }
            }
            _ => {
                if self.disk_controller.accept(address) {
                    let value = self.disk_controller.get_or_set(get, address, value, &self.sender);
                    // println!("Returning byte {:04X}: {:02X}", address, result);
                    result = Some(value);
                }
            }
        }

        if result.is_none() {
            if get {
                let index = if (0x200..0xc000).contains(&address) {
                    if is_set!(self, READ_AUX_MEM_STATUS) { AUX } else { MAIN }
                } else { MAIN };
                result = Some(self.memories[index][address as usize]);
            } else {
                let index = if (0x200..0xc000).contains(&address) {
                    if is_set!(self, WRITE_AUX_MEM_STATUS) { AUX } else { MAIN }
                } else { MAIN };

                self.memories[index][address as usize] = value;
            }
        }

        if result.is_none() && get {
            panic!("None at PC {:04X} address:{:04X}", *PC.read().unwrap(), address);
        }
        // if address == 0xe000 && get && result.unwrap() != 0x4c {
        //     println!("WRONG VALUE FOR E000, PC:{:04X} {:02X}", *PC.read().unwrap(), result.unwrap());
        //     println!();
        // }
        result
    }


    pub(crate) fn load_file(&mut self, file_name: &str, address: u16, skip: u16, max: u16, main: bool) {
        println!("Loading file {}", file_name);
        if let Ok(bytes) = fs::read(file_name) {
            self.load_bytes(&bytes, address, skip, max, main);
        } else {
            println!("Couldn't load file {} in memory, ignoring", file_name);
        }
    }

    pub(crate) fn load_bytes(&mut self, v: &[u8], address: u16, skip: u16, max: u16, main: bool) {
        // println!("Loading {:04X} bytes at address {:04X}", max, address + skip);
        // let v = fs::read(file_name).expect(&format!("Couldn't load file {} in memory", file_name));
        let mut written: u16 = 0;
        let index = if main { 0 } else { 1 };
        // self.memory2.load_bytes(&v, address, skip, max);
        v.iter().enumerate().for_each(|(i, byte)| {
            if i as u16 >= skip && (max == 0 || written < max) {
                let a = address.wrapping_add(i as u16) as usize;
                self.memories[index][a] = *byte;
                written += 1;
            }
        });
    }

    pub(crate) fn dump_at(&self, address: u16, len: u16) {
        hex_dump_at(self.memories[MAIN].as_ref(), address, len);
    }

    fn status(&self, value: bool) -> u8 {
        if value { 0x80 } else { 0 }
    }

    /// Update the F1/F2 switches per patent https://patents.google.com/patent/US4631692
    /// Address is either $C05E or $C05F
    /// F2       F1              Video Mode
    // ______________________________________
    // 0        0               140X192
    // 0        1               160X192
    // 1        0               MIXED_MODE
    // 1        1               560X192
    fn update_f1_f2(&mut self, address: u16) {
        if address == 0xc05f && self.dhg_previous_address == 0xc05e {
            // Shift the register
            self.dhg_rgb_flags = (self.dhg_rgb_flags << 1) & 3;
            self.dhg_rgb_flags |= if is_set!(self, EIGHTY_COLUMNS_STATUS) { 0 } else { 1 };
            self.dhg_rgb_mode = self.dhg_rgb_flags;
            send_message!(&self.sender, RgbModeUpdate(self.dhg_rgb_mode));
        }
        self.dhg_previous_address = address;
    }
}


impl Memory for Apple2Memory {
    fn get(&mut self, address: u16) -> u8 {
        if let Some(v) = self.get_or_set(address, 0, true /* get */) {
            v
        } else {
            0
        }
    }

    fn set(&mut self, address: u16, value: u8) {
        self.get_or_set(address, value, false /* set */);
    }

    fn set_force(&mut self, address: u16, value: u8) {
        self.memories[MAIN][address as usize] = value;
    }

    fn main_memory(&mut self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for i in 0..=0xc5ff {
            result.push(self.memories[MAIN][i]);
        }
        for i in 0xc600..=0xc6ff {
            result.push(self.memories[AUX][i])
        }
        for i in 0xc700..=0xffff {
            // result.push(self.memories[MAIN][i])
            result.push(self.get(i))
        }
        // if result != result2 {
        //     println!("Memories are different");
        // }
        result
    }
}

