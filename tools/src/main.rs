use crate::compress::compress;
use crate::log_analyzer::analyze_logs;

mod log_analyzer;
mod compress;

fn main() {
    compress();
    // analyze_logs();
}