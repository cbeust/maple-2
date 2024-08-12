use std::fs::File;
use std::io::Write;
use crate::constants::CYCLES;

pub(crate) fn increase_cycles(v: u64) {
    let c = CYCLES.read().unwrap().wrapping_add(v);
    *CYCLES.write().unwrap() = c;
}

pub(crate) fn get_cycles() -> u64 {
    *CYCLES.read().unwrap()
}

pub fn bit(v: u8, bit: u8) -> u8 {
    (v & (1 << bit)) >> bit
}

pub fn save(path: &str, buffer: &[u8]) -> Result<(), String> {
    if let Ok(mut file) = File::create(path) {
        file.write_all(buffer).map_err(|e| e.to_string())
    } else {
        Err(format!("Couldn't create {path}"))
    }
}