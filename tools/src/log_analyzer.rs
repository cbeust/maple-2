use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::thread;

#[derive(PartialEq, Eq)]
struct Line {
    line_number: u64,
    absolute_cycles: u64,
    clock: String,
    bit_position: String,
    phase: String,
    direction: String,
    magnet_states: String,
    latch: String,
    address: String,
    bytes: String,
    cycles: String,
    // update_phase: String,
    a: String,
    x: String,
    y: String,
    s: String,
    p: String,
}

const COMPARE_CYCLES: bool = false;
const START: &str = "0801";

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if false {
            f.write_str(&format!(
                "line:{:6}|cycles:{:8} |clock:{:8}|bp:{:8}|phase:{:02}|direction:{:1}| magnetStates:{:1}| latch:{}| {}: {}    ({}) A={} X={} Y={} P={} S={}",
                self.line_number, self.absolute_cycles,
                self.clock, self.bit_position,
                self.phase, self.direction, self.magnet_states, self.latch, // self.update_phase,
                self.address, self.bytes, self.cycles,
                self.a, self.x, self.y, self.p, self.s,
            )).unwrap();
        } else {
            f.write_str(&format!(
                "line:{:6}|cycles:{:8} |{}: {}    ({}) A={} X={} Y={} P={} S={}",
                self.line_number, self.absolute_cycles,
                self.address, self.bytes, self.cycles,
                self.a, self.x, self.y, self.p, self.s,
            )).unwrap();
        }
        Ok(())
    }
}

const MAX: usize = 10_000_000;

pub fn analyze_logs() {
    let (lines_rust, lines_kotlin) = thread::scope(|s| {
        let file_rust = "d:\\t\\trace.csv";
        let file_kotlin = "d:\\t\\trace-kotlin.csv";
        let lines_rust = thread::spawn(|| read_log_csv(file_rust, MAX));
        let lines_kotlin = thread::spawn(|| read_log_csv(file_kotlin, MAX));

        (lines_rust.join().unwrap(), lines_kotlin.join().unwrap())
    });

    for i in 0_usize..MAX {
        if i >= lines_kotlin.len() {
            println!("Reached the end of trace for Kotlin: {}", i);
            break;
        } else if i >= lines_rust.len() {
            println!("Reached the end of trace for Rust: {}", i);
            break;
        } else if ! lines_kotlin[i].is_equal(&lines_rust[i]) {
            println!("Line {} differs:", i);
            println!("Kotlin: {}", lines_kotlin[i]);
            println!("Rust  : {}", lines_rust[i]);
            break;
        }
    }
}

/// Parse "a=b c=d e=f"
fn parse_lines_with_variables(line: &str) -> Vec<(&str, &str)> {
    let mut result: Vec<(&str, &str)> = Vec::new();
    let bindings = line.split(" ");
    // println!("Bindings: {:?}", bindings);
    for binding in bindings {
        // println!("Current binding: {:?}", binding);
        let mut a = binding.split("=");
        let name = a.next().unwrap();
        let value = match a.next() {
            None => { panic!("No value for binding {}", name); }
            Some(v) => { v }
        };
        result.push((name, value))
    }
    result
}

#[derive(Default)]
struct Csv {
    line_number: u128,
    cycles: String,
    pc: String,
    line: String,
    resolved_address: Option<String>,
    resolved_value: Option<String>,
    a: String,
    x: String,
    y: String,
    p: String,
    s: String,
}

impl Csv {
    fn is_equal(&self, other: &Csv) -> bool {
        let result = self.pc == other.pc && self.line == other.line
            && self.resolved_address == other.resolved_address
            && self.resolved_value == other.resolved_value
            && self.a == other.a && self.x == other.x && self.y == other.y
            && self.p == other.p && self.s == other.s;
        result
    }
}

impl Display for Csv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ra = match (&self.resolved_address, &self.resolved_value) {
            (Some(a), Some(v)) => { format!("{}:{}", a, v) }
            (Some(a), None) => { format!("   {}", a) }
            (None, Some(v)) => { format!("     {}", v) }
            _ => { "       ".to_string() }
        };
        f.write_str(&format!("{}: {}| {} {ra} {} {} {} {} {} {}",
            self.line_number, self.cycles, self.line, ra, self.a, self.x, self.y, self.p, self.s)).unwrap();
        Ok(())
    }
}

fn read_log_csv(filename: &str, _max: usize) -> Vec<Csv> {
    println!("Reading {filename}");
    let mut result: Vec<Csv> = Vec::new();
    let mut stop = false;
    let all_lines = read_to_string(filename).unwrap();
    let mut iter = all_lines.lines().into_iter();
    let mut line_number = 0_u128;

    while !stop {
        let line = if let Some(l) = iter.next() {
            l
        } else {
            stop = true;
            continue;
        };

        if let Some(c) = line.chars().next() {
            if c.is_ascii_digit() {
                let mut csv = Csv::default();
                let mut elements = line.split(",");
                csv.line_number = line_number;
                // skip cycles
                let _ = elements.next().unwrap();
                csv.cycles = "0".to_string(); // elements.next().unwrap().to_string();
                csv.pc = elements.next().unwrap().to_string();
                csv.line = elements.next().unwrap().to_string();
                csv.resolved_address = elements.next().map(|s| s.to_string());
                csv.resolved_value = elements.next().map(|s| s.to_string());
                csv.a = elements.next().unwrap().to_string();
                csv.x = elements.next().unwrap().to_string();
                csv.y = elements.next().unwrap().to_string();
                csv.p = elements.next().unwrap().to_string();
                csv.s = elements.next().unwrap().to_string();
                result.push(csv);
            }
            line_number += 1;
        }
    }
    result
}

fn read_kotlin(filename: &str, max: usize) -> Vec<Line> {
    println!("Reading {filename}");
    let mut result: Vec<Line> = Vec::new();
    let mut line_number: u64 = 0;
    let mut stop = false;
    let all_lines = read_to_string(filename).unwrap();
    let mut iter = all_lines.lines().into_iter();
    let mut has_started = false;
    let mut clock = "".to_string();
    let mut bit_position = "".to_string();
    let mut logged_direction = "".to_string();
    let mut logged_phase = "".to_string();
    let mut logged_magnet_states = "".to_string();
    let mut logged_latch = "".to_string();
    let mut logged_update_phase = "".to_string();
    while ! stop {
        let line = if let Some(l) = iter.next() {
            l
        } else {
            stop = true;
            continue;
        };

        if line.contains("@@") {
            let mut s = line.split("@@ ");
            s.next().unwrap();
            let bindings = parse_lines_with_variables(s.next().unwrap());
            for binding in bindings {
                let value = binding.1.to_string();
                match binding.0 {
                    "clock" => { clock = value;  }
                    "bitPosition" => { bit_position = value; }
                    "direction" => { logged_direction = value; }
                    "phase" => { logged_phase= value; }
                    "magnetStates" => { logged_magnet_states = value; }
                    "latch" => { logged_latch = value; }
                    "updatePhase" => { logged_update_phase = value; }
                    _ => { panic!("Unknown log variable: {}", binding.0); }
                }
            }
            continue;
        }

        let c = match line.chars().next() {
            None => { continue; }
            Some(ch) => { ch }
        };

        let ok = c.is_digit(10) || c == 'A' || c == 'B' || c == 'C' || c == 'D' || c == 'E'
            || c == 'F';

        if ! ok {
            // println!("Skipping bogus {line}");
            continue;
        }

        let mut elements = line.trim().split(" ");
        let ac = elements.next().unwrap();
        let l = ac.len();
        let mut absolute_cycles_s = ac[0..l - 1].to_string();
        let absolute_cycles = if COMPARE_CYCLES {
            match absolute_cycles_s.parse::<u64>() {
                Ok(n) => { n }
                Err(_) => {
                    continue;
                }
            }
        } else {
            0
        };
        let address = elements.next().unwrap().trim().split(":").next().unwrap();
        let mut bytes: Vec<String> = Vec::new();
        let b = match elements.next() {
            None => {
                println!("Error parsing line {}", line);
                break;
            }
            Some(e) => e
        };
        if b.len() == 2 {
            bytes.push(b.to_string());
        }
        let b = elements.next().expect(&format!("Error at line {}", line_number));
        if b.len() == 2 {
            bytes.push(b.to_string());
            let b = elements.next().unwrap();
            if b.len() == 2 {
                bytes.push(b.to_string());
            }
        }
        let bytes_string = bytes.join(" ");

        let mut a = "".to_string();
        let mut x = "".to_string();
        let mut y = "".to_string();
        let mut p = "".to_string();
        let mut ss = "".to_string();
        let mut cycles = "".to_string();
        while let Some(s) = elements.next() {
            if s.starts_with("A=") {
                a = s[2..=3].to_string();
            }
            if s.starts_with("X=") {
                x = s[2..=3].to_string();
            }
            if s.starts_with("Y=") {
                y = s[2..=3].to_string();
            }
            if s.starts_with("P=") && ! s.starts_with("P=$") {
                p = s[2..=3].to_string();
            }
            if s.starts_with("S=") {
                ss = s[2..=3].to_string();
            }
            if s.starts_with("(") && s.ends_with(")") {
                cycles = s[1..=1].to_string();
            }
        }

        if ! has_started && address == START {
            has_started = true;
        }

        if has_started {
            if a == "" {
                println!("Should not happen: {}", line_number);
            }
            result.push(Line {
                absolute_cycles,
                line_number,
                clock: clock.to_string(),
                bit_position: bit_position.to_string(),
                address: address.to_string(),
                bytes: bytes_string,
                latch: logged_latch.to_string(),
                cycles,
                a,
                x,
                y,
                p,
                s: ss,
                direction: logged_direction.to_string(),
                phase: logged_phase.to_string(),
                magnet_states: logged_magnet_states.to_string(),
                // update_phase: logged_update_phase.to_string(),
            });
            line_number += 1;
            stop = line_number as usize >= max;
        }
    }
    println!("Done reading {filename}, {} lines", line_number);
    result
}