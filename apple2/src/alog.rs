use std::time::Instant;
use tracing::info;
use crate::constants::{CYCLES, PC, START};

//#[cfg(test)]
pub fn alog(s: &str) {
    let elapsed = (Instant::now() - *START.get().unwrap()).as_millis();
    info!("cycles:{} ms:{} PC:{:04X}| {}", *CYCLES.read().unwrap(), elapsed,
        *PC.read().unwrap(), s);
}