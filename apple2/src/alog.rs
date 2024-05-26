use std::time::Instant;
use crate::constants::START;

#[cfg(test)]
pub fn alog(s: &str) {
    let elapsed = (Instant::now() - *START.get().unwrap()).as_millis();
    log::info!("{}| {}", elapsed, s);
}