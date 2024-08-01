use crate::log_analyzer::analyze_logs;

pub mod log_analyzer;
mod compress;
mod csv;

fn main() {
    // compress();
    analyze_logs();
}