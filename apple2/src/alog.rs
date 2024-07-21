use std::time::Instant;
use crate::constants::{PC, START};

//#[cfg(test)]
pub fn alog(s: &str) {
    let elapsed = (Instant::now() - *START.get().unwrap()).as_millis();
    log::info!("c:{}| PC:{:04X}| {}", elapsed, *PC.read().unwrap(), s);
}