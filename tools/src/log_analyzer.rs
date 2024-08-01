use std::fmt::{Display, Formatter};
use std::fs::{File, read_to_string};
use std::io::{BufRead, BufReader,};
use std::{io};
use crate::csv::{Csv, Ignore};

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

pub fn analyze_logs() -> io::Result<()> {
    let file_me = "c:\\t\\trace-temp.txt";
    let file_other = "d:\\Apple Disks\\Trace-temp.txt";
    // let lines_other = read_log(file_other, MAX);

    let mut lines_me = BufReader::new(File::open(file_me)?).lines();
    let mut lines_other = BufReader::new(File::open(file_other)?).lines();

    let mut stop = false;
    let mut line_count = 0;
    let mut ignore_a = false;
    let mut ignore_x = false;
    let mut ignore_y = false;
    while !stop {
        let (lines_skipped_me, csv_me) = Csv::parse(&mut lines_me);
        let (lines_skipped_other, csv_other) = Csv::parse(&mut lines_other);
        if (line_count % 1000) == 0 {
            println!("Line: {line_count}");
        }
        match (csv_me, csv_other) {
            (Some(c1), Some(c2)) => {
                if ! c1.is_equal(&c2, ignore_a, ignore_x, ignore_y) {
                    println!("Differ at line {} / {}",
                        line_count + lines_skipped_me, line_count + lines_skipped_other);
                    println!("Me:      {}", c1.full_line);
                    println!("Other:   {}", c2.full_line);

                    stop = true;
                }
                match c1.calculate_ignore() {
                    Ignore::DoIgnore => { ignore_a = true; }
                    Ignore::DontIgnore => { ignore_a = false;}
                    Ignore::DontChange => {}
                }
                // println!("Current ignore: A:{ignore_a}");
            }
            (None, Some(_)) => {
                println!("My file ran out of lines");
                stop = true;
            }
            (Some(_), None) => {
                println!("Other file ran out of lines");
                stop = true;
            }
            _ => {
                println!("Both files are done");
                stop = true;
            }
        }
        line_count += 1;
        // match (lines_me.next(), lines_other.next()) {
        //     (Ok(l1), Ok(l2)) => {}
        //     _ => {
        //         stop = true;
        //     }
        // }
    }
    Ok(())
}

    // let (lines_me, lines_other) = thread::scope(|s| {
    //     let lines_me = thread::spawn(|| read_log(file_me, MAX));
    //     let lines_other = thread::spawn(|| read_log(file_other, MAX));
    //
    //     (lines_me.join().unwrap(), lines_other.join().unwrap())
    // });
    //
    // for i in 0_usize..MAX {
    //     if i >= lines_other.len() {
    //         println!("Reached the end of trace for other: {}", i);
    //         break;
    //     } else if i >= lines_me.len() {
    //         println!("Reached the end of trace for me: {}", i);
    //         break;
    //     } else if ! lines_other[i].is_equal(&lines_me[i]) {
    //         println!("Line {} differs:", i);
    //         println!("Kotlin: {}", lines_other[i]);
    //         println!("Rust  : {}", lines_me[i]);
    //         break;
    //     }
    // }

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

// /// Format:
// /// 04187E08 01 60 00 01FC ..RB.I.C  0801:86 0A     STX $0A
// pub fn read_log_applewin(filename: &str, _max: usize) -> Vec<Csv> {
//     let mut result = Vec::new();
//
//     let all_lines = read_to_string(filename).unwrap();
//     let mut iter = all_lines.lines().into_iter();
//     let mut line_number = 0_u128;
//     let mut stop = false;
//     let mut pc = "XXXX".to_string();
//     let mut first = true;
//
//     while ! stop {
//         let line = if let Some(l) = iter.next() {
//             l
//         } else {
//             stop = true;
//             continue;
//         };
//
//         let mut it = line.split(' ').into_iter();
//         let cycles = it.next().unwrap();
//         let a = format!("A={}", it.next().unwrap());
//         let x = format!("X={}", it.next().unwrap());
//         let y = format!("Y={}", it.next().unwrap());
//         let _ = it.next().unwrap();  // skip stack, starts at $1FC for some reason
//         // Format: ..RB.I.C
//         let flags_string = it.next().unwrap();
//         let mut p = 0;
//         for c in flags_string.chars() {
//             p <<= 1;
//             if c != 'I' && c != '.' { p |= 1 };
//         }
//         let p = format!("P={p:02X}");
//
//         // AppleWin defines the registers for the previous PC, so add the entry now
//         // and save the PC for the next iteration
//         if ! first {
//             result.push( Csv {
//                 full_line: "blah".to_string(),
//                 line_number,
//                 cycles: "".to_string(),
//                 pc,
//                 ops: "".to_string(),
//                 resolved_address: None,
//                 resolved_value: None,
//                 a,
//                 x,
//                 y,
//                 p,
//                 s: "".to_string(),
//             });
//         }
//         first = false;
//
//         // Read PC
//         let mut pc_opcode = it.next().unwrap();
//         pc_opcode = it.next().unwrap();
//         pc = pc_opcode.split(':').next().unwrap().to_string();
//         // println!("Flags: {} becomes {p}", flags_string);
//         // println!("Read A:{a} PC_OPCODE:{pc_opcode} PC:{pc}");
//         // println!("");
//         line_number += 1;
//     }
//
//     result
// }


// // 02248039 D0 60 07 01FF ...B.I.C B700: 8E E9 B7   STX $B7E9     |  A$B7E9:v$60
// fn read_log(filename: &str, _max: usize) -> Vec<Csv> {
//     println!("Reading {filename}");
//     let mut result: Vec<Csv> = Vec::new();
//     let mut stop = false;
//     let all_lines = read_to_string(filename).unwrap();
//     let mut iter = all_lines.lines().into_iter();
//     let mut line_number = 0_u128;
//
//     let mut line_count = 0;
//     while !stop {
//         let line = if let Some(l) = iter.next() {
//             l
//         } else {
//             stop = true;
//             continue;
//         };
//
//         // let line = "11FDB07B 45 4B B6 01DF ..RB.I.C  F8C9:B9 00 FB  LDA $FB00,Y";
//         //
//         //
//         // let re2 = Regex::new("([[:xdigit:]]+) ([[:xdigit:]]{2}) ([[:xdigit:]]{2}) ([[:xdigit:]]{2}) \
//         //             ([[:xdigit:]]{4}) +([^ ]{8}) +([[:xdigit:]]{4}): +((?:.. )+)")
//         //     .unwrap();
//         // let s3 =             "11FDB07B 45 4B B6 01DF ..RB.I.C  F8C9:B9 00 FB  LDA $FB00,Y";
//         // let m = re2.is_match("11FDB07B 12 34 56 01DF ..RB.I.C  F8C9: 12 34 56  LDA $FB00,Y");
//         // if m {
//         //     println!("True");
//         // } else {
//         //     println!("False");
//         // }
//         //
//         //
//         // if ! re.is_match(line) {
//         //     println!("Not matching:\n{line}");
//         //     println!("");
//         // }
//         for (_, [cycles, a, x, y, stack, flags, pc, op, rest]) in RE.captures_iter(line).map(|c| c.extract()) {
//             // println!("Cycles: {cycles} A:{a} X:{x} Y:{y} stack:{stack} flags:{flags} pc:{pc} op:{op} rest:{rest}");
//
//             if line_count != 0 && (line_count % 100_000) == 0 {
//                 println!("Line {line_count}");
//             }
//             let mut csv = Csv::default();
//             csv.cycles = cycles.into();
//             csv.pc = pc.into();
//             csv.a = a.into();
//             csv.x = x.into();
//             csv.y = y.into();
//             csv.p = flags.into();
//             csv.s = stack.into();
//             csv.ops = op.trim().to_string();
//             result.push(csv);
//             line_count += 1;
//         }
//     }
//     result
// }

// fn read_log_csv(filename: &str, _max: usize) -> Vec<Csv> {
//     println!("Reading {filename}");
//     let mut result: Vec<Csv> = Vec::new();
//     let mut stop = false;
//     let all_lines = read_to_string(filename).unwrap();
//     let mut iter = all_lines.lines().into_iter();
//     let mut line_number = 0_u128;
//
//     while !stop {
//         let line = if let Some(l) = iter.next() {
//             l
//         } else {
//             stop = true;
//             continue;
//         };
//
//         if let Some(c) = line.chars().next() {
//             if c.is_ascii_digit() {
//                 let mut csv = Csv::default();
//                 let mut elements = line.split('_');
//                 csv.line_number = line_number;
//                 // skip cycles
//                 let _ = elements.next().unwrap();
//                 csv.cycles = "0".to_string(); // elements.next().unwrap().to_string();
//                 csv.pc = elements.next().unwrap().to_string();
//                 csv.ops = elements.next().unwrap().trim().to_string();
//                 csv.resolved_address = elements.next().map(|s| s.to_string());
//                 csv.resolved_value = elements.next().map(|s| s.to_string());
//                 csv.a = elements.next().unwrap().to_string();
//                 csv.x = elements.next().unwrap().to_string();
//                 csv.y = elements.next().unwrap().to_string();
//                 csv.p = elements.next().unwrap().to_string();
//                 csv.s = elements.next().unwrap().to_string();
//                 result.push(csv);
//             }
//             line_number += 1;
//         }
//     }
//     result
// }

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