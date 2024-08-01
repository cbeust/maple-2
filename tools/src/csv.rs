use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufReader, Lines};
use once_cell::unsync::Lazy;
use regex::Regex;

const RE: Lazy<Regex> = Lazy::new(|| Regex::new(concat!(
    "([[:xdigit:]]+) ",  // cycles
    "([[:xdigit:]]{2}) ([[:xdigit:]]{2}) ([[:xdigit:]]{2}) ", // A X Y
    "([[:xdigit:]]{4}) +", // stack
    "([^ ]{8}) +", // flags
    "([[:xdigit:]]{4}): *", // PC
    "((?:.. )+)",
    // "(([[:xdigit:]]{2}) )*  ", // op
    "(.+)$"
    )).unwrap());


#[derive(Default)]
pub struct Csv {
    pub full_line: String,
    line_number: u128,
    cycles: String,
    pc: String,
    ops: String,
    resolved_address: Option<String>,
    resolved_value: Option<String>,
    a: String,
    x: String,
    y: String,
    p: String,
    s: String,
}

pub enum Ignore {
    DoIgnore,
    DontIgnore,
    DontChange,
}

impl Csv {
    /// Read lines from the iterator until we either reach one that can successfully
    /// parse into a Csv or we reach the end of file
    /// Return the number of lines skipped and an optional Csv
    pub fn parse(it: &mut Lines<BufReader<File>>) -> (usize, Option<Csv>) {
        let mut stop = false;
        let mut lines_skipped = 0;
        let mut result: Option<Csv> = None;
        while ! stop {
            match it.next() {
                None => {
                    println!("End of file reached");
                    stop = true;
                }
                Some(s) => {
                    match s {
                        Ok(line) => {
                            if RE.is_match(&line) {
                                if let Some(csv) = Self::to_csv(&line) {
                                    if let Ok(pc) = u16::from_str_radix(&csv.pc, 16) {
                                        if pc < 0xc000 {
                                            result = Some(csv);
                                            stop = true;
                                        } else {
                                            // println!("Ignoring PC: {pc:04X}");
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("Err: {e}");
                        }
                    }
                }
            }
            lines_skipped += 1;
        }

        (lines_skipped, result)
    }

    pub fn calculate_ignore(&self) -> Ignore {
        let ops = self.ops.split(" ")
            .map(|s| match u16::from_str_radix(s, 16) {
                Ok(hex) => { Some(hex) }
                Err(_) => { None }
            })
            .filter_map(|s| s)
            .collect::<Vec<u16>>()
        ;

        let mut result = Ignore::DontChange;
        let is_floating =
            (ops.len() == 2 && ops[1] == 0x4e) ||
            (ops.len() == 3 && ops[2] == 0xc0);

        if ops.len() == 3 {
            // Loading from C08x
            match ops[0] {
                0x2c |   // BIT
                0xad | 0xa9 | 0xa5 | 0xb5 | 0xd | 0xbd | 0xb9 | 0xa1 | 0xb1=> {
                    result = if is_floating { Ignore::DoIgnore } else { Ignore::DontIgnore };
                }
                _ => {}
            }
        }

        result
    }

    fn to_csv(s: &str) -> Option<Csv> {
        match RE.captures(&s) {
            None => {
                println!("Skipping non matching line {s}");
                println!("");
                None
            }
            Some(c) => {
                let (_, [cycles, a, x, y, stack, flags, pc, op, rest]) = c.extract();
                let mut csv = Csv::default();
                csv.full_line = s.to_string();
                csv.cycles = cycles.into();
                csv.pc = pc.into();
                csv.a = a.into();
                csv.x = x.into();
                csv.y = y.into();
                csv.p = flags.into();
                csv.s = stack.into();
                csv.ops = op.trim().to_string();
                csv.calculate_ignore();
                Some(csv)
            }
        }
    }

    // fn is_equal_applewin(&self, other: &Csv) -> bool {
    //     if self.pc != other.pc {
    //         println!("PC");
    //     }
    //     if self.a != other.a {
    //         println!("A");
    //     }
    //     if self.x != other.x {
    //         println!("X");
    //     }
    //     if self.y != other.y {
    //         println!("Y");
    //     }
    //     if self.p != other.p {
    //         println!("P");
    //     }
    //     let result = self.pc == other.pc
    //         && self.a == other.a && self.x == other.x && self.y == other.y
    //         // && self.p == other.p
    //         ;
    //     result
    // }

    pub fn is_equal(&self, other: &Csv, ignore_a: bool, ignore_x: bool, ignore_y: bool) -> bool {
        let result = self.pc == other.pc // && self.line == other.line
            // && self.resolved_address == other.resolved_address
            // && self.resolved_value == other.resolved_value
            && (ignore_a || self.a == other.a)
            && self.x == other.x
            && self.y == other.y
            && (ignore_a || self.p == other.p)
            // && self.s == other.s
            ;
        if ! result {
            if self.pc != other.pc {
                println!("Different PC");
            }
            if ! ignore_a && self.a != other.a {
                println!("Different A");
            }
            if self.x != other.x {
                println!("Different x");
            }
            if self.y != other.y {
                println!("Different Y");
            }
        }
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
            self.line_number, self.cycles, self.ops, ra, self.a, self.x, self.y, self.p, self.s)).unwrap();
        Ok(())
    }
}

