use std::fs::File;
use std::io::Write;
use eframe::egui::{Color32, Pos2, Rect, Stroke, Ui};
use crate::constants::CYCLES;

pub fn rect(ui: &mut Ui, x1: f32, y1: f32, x2: f32, y2: f32, color: Color32) {
    ui.painter().rect_stroke(Rect::from_points(&[Pos2::new(x1, y1), Pos2::new(x2, y2)]),
         0.0,
         Stroke {
             width: 1.0,
             color,
         });
}

pub(crate) fn increase_cycles(v: u128) {
    let c = CYCLES.read().unwrap().wrapping_add(v);
    *CYCLES.write().unwrap() = c;
}

pub(crate) fn get_cycles() -> u128 {
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