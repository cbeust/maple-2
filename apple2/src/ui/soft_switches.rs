use std::string::ToString;
use lazy_static::lazy_static;
use crate::memory_constants::*;

pub struct SoftSwitch {
    pub name: String,
    pub on: u16,
    pub off: u16,
    status: u16,
}

impl SoftSwitch {
    fn new(name: &str, on: u16, off: u16, status: u16) -> Self {
        Self { name: name.to_string(), on, off, status }
    }

    pub fn is_set(&self, memory: &Vec<u8>) -> bool {
        memory[self.status as usize] >= 0x80
    }

    pub fn flip(&self, memory: &mut Vec<u8>) {
        let new_status = if self.is_set(memory) {
            0
        } else {
            0x80
        };
        memory[self.status as usize] = new_status;
    }
}

lazy_static! {
    pub static ref SOFT_SWITCHES: Vec<SoftSwitch> = vec![
        SoftSwitch::new("Text", TEXT_ON, TEXT_OFF, TEXT_STATUS),
        SoftSwitch::new("Mixed", MIXED_ON, MIXED_OFF, MIXED_STATUS),
        SoftSwitch::new("Page2", PAGE_2_ON, PAGE_2_OFF, PAGE_2_STATUS),
        SoftSwitch::new("Hi Res", HIRES_ON, HIRES_OFF, HIRES_STATUS),

        SoftSwitch::new("Store80", EIGHTY_STORE_ON, EIGHTY_STORE_OFF, EIGHTY_STORE_STATUS),
        SoftSwitch::new("Read aux", READ_AUX_MEM_ON, READ_AUX_MEM_OFF, READ_AUX_MEM_STATUS),
        SoftSwitch::new("Write aux", WRITE_AUX_MEM_ON, WRITE_AUX_MEM_OFF, WRITE_AUX_MEM_STATUS),
        SoftSwitch::new("Alt ZP", ALT_ZP_ON, ALT_ZP_OFF, ALT_ZP_STATUS),
        SoftSwitch::new("Slot C3", SLOT_C3_ON, SLOT_C3_OFF, SLOT_C3_STATUS),
        SoftSwitch::new("80 Columns", EIGHTY_COLUMNS_ON, EIGHTY_COLUMNS_OFF, EIGHTY_COLUMNS_STATUS),
        SoftSwitch::new("Alt Char", ALT_CHAR_ON, ALT_CHAR_OFF, ALT_CHAR_STATUS),
    ];
}
