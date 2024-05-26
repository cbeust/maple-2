mod ui;
mod list_management;

use std::fmt::{Debug, Display, Formatter};
use std::{env, fs, thread};
use std::string::ToString;
use std::sync::mpsc::{Receiver, Sender};
use clap::Parser;
use serde::Deserialize;
use lazy_static::lazy_static;
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

use cpu::memory::DefaultMemory;
use cpu::cpu::{Cpu, RunStatus, StatusFlags};

// const HARTE_DIRECTORY: &str = "d:\\pd\\ProcessorTests\\6502\\v1\\";
const HARTE_DIRECTORY: &str = "c:\\Users\\Ced\\t\\ProcessorTests\\rockwell65c02\\v1";
const FIRST_TEST: usize = 0x0;
// Bug in SBC (0xe1) in BCD mode

lazy_static! {
    // Unimplemented opcodes that we don't support yet
    // From https://www.masswerk.at/6502/6502_instruction_set.html
    static ref SKIPPED_6502: Vec<u8> = vec![
        0x6b, // ARR
        0x8b, // ANE, not sure how Harte implements it
        0x93, // SHA
        0x9b, // TAS
        0x9c, // SHY
        0x9e, // SHX
        0x9f, // SHA (AHX, AXA)
        0xab, // LXA
        0xcb, // SBX
    ];

    static ref SKIPPED_65C02: Vec<u8> = Vec::new(); //OPCODES_65C02.to_vec();

    // static ref SKIPPED: &Vec<u8> = SKIPPED_65C02;

    static ref SKIPPED_TESTS: Vec<String> = {
        let mut s: Vec<String> = Vec::new();
        for op in SKIPPED_65C02.iter() {
            s.push(format!("{:02x}", op));
        }
        s
    };
}

#[derive(clap_derive::Parser, Default, Debug)]
#[clap(author)] // , trailing_var_arg=true)]
struct Args {
    #[arg(short, long)]
    text_only: bool,

    #[arg(short, long)]
    skip_cycles: bool,

    #[arg(short, long)]
    skip_bcd: bool,

    rest: Vec<String>,
}

pub fn main() -> Result<(), ()> {
    // let test_name = "e1 d2 0d".to_string();
    // let test_name = "02 58 f3".to_string();
    // let file_name = format!("{}.json", test_name.split(" ").next().unwrap());
    // let result = run_one_specific_test(&HARTE_DIRECTORY, &file_name, &test_name);
    // println!("Result for test {}: {:?}", test_name, result);

    let args = Args::parse();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(log::LevelFilter::Info))
            .unwrap();

    log4rs::init_config(config).unwrap();

    if args.rest.is_empty() {
        //
        // No argument: display the UI
        //
        let (sender, receiver) = std::sync::mpsc::channel::<FileStatus>();
        thread::scope(|s| {
            s.spawn(|| ui::main(receiver));
            s.spawn(|| {
                let files = read_directory(HARTE_DIRECTORY);
                run_files(files, &args, sender);
            });
        });
    } else {

        //
        // Run either a whole file (e.g. 01) or a specific test (e.g. 01 2a ff)
        //
        let (sender, receiver) = std::sync::mpsc::channel::<FileStatus>();
        if args.text_only {
            // text mode
            thread::scope(|s| {
                s.spawn(|| {
                    text_loop(receiver);
                });
                s.spawn(|| {
                    let files = read_directory(HARTE_DIRECTORY);
                    run_files(files, &args, sender.clone());
                });
            });
        } else {
            let test_name = &args.rest[0];
            if args.rest.len() == 1 {
                // Run e.g. 01.json
                let mut files: Vec<String> = Vec::new();
                let file = format!("{}\\{}.json", HARTE_DIRECTORY, test_name);
                println!("Running tests in {}", file);
                files.push(file);
                run_files(files, &args, sender);
                text_loop(receiver);
            } else {
                // Run a specific test (e.g. 12 aa fe)
                let file_name = format!("{:02}.json", test_name);
                let full_test_name = args.rest.join(" ");
                // let mut full_test_name = test_name.clone();
                // while let Some(arg) = args.next() {
                //     full_test_name.push_str(" ");
                //     full_test_name.push_str(&arg);
                // }
                // let test_name = format!("{} {} {}", test_name, args.next().unwrap(), args.next().unwrap());
                let result = run_one_specific_test(&HARTE_DIRECTORY, &file_name, &full_test_name, &args);
                match result {
                    Ok(test_status) => {
                        println!("== Test {}", test_status.name());
                        match test_status {
                            TestStatus::Passed(_, _) => {
                                println!("=== Passed");
                            }
                            TestStatus::Failed(_, _, these_errors) => {
                                println!("=== Failed: {:?}", these_errors);
                            }
                            TestStatus::Skipped(_, _, _) => {
                                println!("=== Skipped");
                            }
                        };
                    }
                    Err(_) => { panic!("Couldn't find test {}", test_name) }
                };
            }
        }
    }

    Ok(())
}

fn text_loop(receiver: Receiver<FileStatus>) {
    loop {
        match receiver.recv() {
            Ok(test) => {
                match test {
                    FileStatus::NotStarted(_) => {}
                    FileStatus::Completed(test, passed, failed, skipped, statuses) => {
                        if failed > 0 {
                            println!("{:02X} Passed: {}, failed: {}, skipped: {}",
                                     test, passed, failed, skipped);
                        }
                        if failed > 0 {
                            let max = std::cmp::min(statuses.len(), 5);
                            for status in &statuses[0..max] {
                                match status {
                                    TestStatus::Passed(_, _) => {}
                                    TestStatus::Skipped(_, _, _) => {}
                                    TestStatus::Failed(_, name, errors) => {
                                        println!("  {}:", name);
                                        errors.iter().for_each(|error|
                                            println!("     {}", error));
                                    }
                                }
                            }
                        }
                    }
                    FileStatus::Exit() => { break }
                }
            }
            Err(_) => { break }
        }
    }
}

#[derive(Debug, Deserialize)]
struct State {
    pc: u16,
    s: u8,
    a: u8,
    x: u8,
    y: u8,
    p: u8,
    ram: Vec<MemValue>,
}

#[derive(Debug, Deserialize)]
struct MemValue {
    address: u16,
    value: u8,
}

#[derive(Debug, Deserialize)]
struct Cycle {
    address: usize,
    value: u8,
    access_type: String,
}

#[derive(Debug, Deserialize)]
struct Test {
    /// e.g. e7 a0 22
    name: String,
    initial: State,
    #[serde(rename = "final")]
    final_state: State,
    cycles: Vec<Cycle>
}

impl Test {
    fn extract_opcode(&self) -> u8 {
        let num_str = &self.name[0..2];
        usize::from_str_radix(num_str, 16).unwrap() as u8
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        _ = f.write_str(&format!("Expected cycle count: {}\n", self.cycles.len()));
        _ = f.write_str(&format!("Initial registers:  A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{:02X} PC:{:04X} {}\n",
                             self.initial.a, self.initial.x, self.initial.y,
                             self.initial.s, self.initial.p, self.initial.pc,
                             StatusFlags::new_with(self.initial.p)));
        _ =f.write_str(&format!("Expected registers: A:{:02X} X:{:02X} Y:{:02X} S:{:02X} P:{:02X} PC:{:04X} {}\n",
                             self.final_state.a, self.final_state.x, self.final_state.y,
                             self.final_state.s, self.final_state.p, self.final_state.pc,
                             StatusFlags::new_with(self.final_state.p)));
        let mut initial: Vec<&MemValue> = self.initial.ram.iter().collect();
        initial.sort_by(|a, b| a.address.partial_cmp(&b.address).unwrap());
        _ = f.write_str("Initial memory:  ");
        for mv in &initial {
            _ = f.write_str(&format!("{:04X}:{:02X} ", mv.address, mv.value));
        }
        _ =f.write_str("\nExpected memory: ");
        let mut final_state: Vec<&MemValue> = self.final_state.ram.iter().collect();
        final_state.sort_by(|a, b| a.address.partial_cmp(&b.address).unwrap());
        for mv in &final_state {
            _ = f.write_str(&format!("{:04X}:{:02X} ", mv.address, mv.value));
        }
        Ok(())
    }
}

// impl Debug for Test {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.write_str(&*format!("Test name: {}", self.name));
//         Ok(())
//     }
// }


fn read_directory(directory: &str) -> Vec<String> {
    let entries = fs::read_dir(directory).unwrap().into_iter();
    let mut files: Vec<String> = Vec::new();
    for file in entries {
        let file = file.unwrap();
        let name = file.path().to_owned();
        let full_name = name.to_str().unwrap();

        let file_name = file.file_name();
        let file_name = file_name.to_str().unwrap();
        let num_str = file_name.split(".").next().unwrap();
        let number = usize::from_str_radix(num_str, 16).unwrap();
        if number >= FIRST_TEST {
            files.push(full_name.to_string());
        }
    }

    files.sort();
    files
}

pub enum FileStatus {
    // opcode
    NotStarted(u8),
    // opcode, passed, failed, skipped, failed_tests
    Completed(u8, u64, u64, u64, Vec<TestStatus>),
    Exit(),
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed(u8, String),
    Skipped(u8, String, String),  // test name, reason
    Failed(u8, String, Vec<String>), // test name, errors
}

impl TestStatus {
    fn name(&self) -> String {
        match self {
            TestStatus::Passed(_, name) => { name }
            TestStatus::Failed(_, name, _) => { name }
            TestStatus::Skipped(_, name, _) => { name }

        }.to_string()
    }
}

fn extract_test_number(file_name: &str) -> String {
    let mut file = file_name.split("\\");
    if let Some(f) = file.find(|c| c.contains(".")) {
        let mut c = f.split(".");
        if let Some(f2) = c.next() {
            return f2.to_string();
        }
    }
    panic!("Couldn't extract test name from {}", file_name);
}

fn run_files(files: Vec<String>, arguments: &Args, sender: Sender<FileStatus>) -> Vec<TestStatus> {
    let mut result: Vec<TestStatus> = Vec::new();
    use rayon::prelude::*;
    // let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    files.iter()
        .par_bridge()
        .for_each_with(sender, |sender, name| {
            let _ = run_one_file(name, arguments, sender.clone());
        });
    result
}


fn is_skipped(test: &Test) -> bool {
    for skipped in SKIPPED_TESTS.iter() {
        let name = &test.name;
        if name.starts_with(skipped) {
            return true;
        }
    }
    false
}

fn run_one_file(file_name: &str, arguments: &Args, sender: Sender<FileStatus>) {
    let mut result: Vec<TestStatus> = Vec::new();
    let data = fs::read_to_string(file_name).expect("Unable to read file");
    let tests: Vec<Test> = serde_json::from_str(&data).expect("Unable to parse");
    let test_number = extract_test_number(file_name);
    let test_number = u8::from_str_radix(&test_number, 16).unwrap();

    let mut passed = 0_u64;
    let mut failed = 0_u64;
    let mut skipped = 0_u64;
    let mut failed_tests: Vec<TestStatus> = Vec::new();
    for test in tests.iter() {
        if is_skipped(&test) {
            skipped += 1;
        } else {
            let test_status = run_one_test(test, arguments, false);
            let ts2 = test_status.clone();
            match test_status {
                TestStatus::Passed(_, _) => { passed += 1; }
                TestStatus::Skipped(_, _, _) => { skipped += 1; }
                TestStatus::Failed(_, _, _) => {
                    failed_tests.push(ts2);
                    failed += 1;
                }
            }
            result.push(test_status);
        }
    }
    // log::info!("Completed file {}", file_name);
    _ = sender.send(FileStatus::Completed(test_number, passed, failed, skipped, failed_tests));
}

fn run_one_specific_test(directory: &str, file_name: &str, test_name: &str, args: &Args)
        -> Result<TestStatus, String> {
    let fq = format!("{}\\{}", directory, file_name);
    let data = fs::read_to_string(fq).expect("Unable to read file");
    let test: Vec<Test> = serde_json::from_str(&data).expect("Unable to parse");
    let result = if let Some(test) = test.iter().find(|test| test.name == test_name) {
        println!("Running:\n{}", &test);
        Ok(run_one_test(test, args, true))
    } else {
        Err(format!("Couldn't find test named {} in file {}", test_name, file_name))
    };

    result
}

fn run_one_test(test: &Test, args: &Args, debug_asm: bool) -> TestStatus {
    let bcd = (test.initial.p & 8) > 0;
    if bcd && args.skip_bcd {
        return TestStatus::Skipped(test.extract_opcode(), test.name.clone(),
            "BCD is being ignored".to_string());
    }

    //
    // Initialize the memory
    //
    use cpu::memory::Memory;
    let mut memory = DefaultMemory::new();
    for mem_value in test.initial.ram.iter() {
        // println!("Setting memory {:04X}:{:02X}", mem_value.address, mem_value.value);
        memory.set(mem_value.address, mem_value.value);
    }

    //
    // Initialize the registers
    //
    let mut cpu = Cpu::new(memory, cpu::config::Config::default());
    cpu.asm_always = debug_asm;
    cpu.pc = test.initial.pc;
    cpu.a = test.initial.a;
    cpu.x = test.initial.x;
    cpu.y = test.initial.y;
    cpu.s = test.initial.s;
    cpu.p.set_value(test.initial.p);
    // println!("CPU: {}", cpu);

    //
    // Run one step
    //
    let config = cpu::config::Config::default();
    let test_status = cpu.step(&config);
    let cycles = match test_status {
        RunStatus::Continue(c) => { c }
        RunStatus::Stop(_, _, _) => { 0 }
    };

    let mut errors: Vec<String> = Vec::new();

    //
    // Verify cycles
    //
    if ! args.skip_cycles && cycles != test.cycles.len() as u8 {
        errors.push(format!("Wrong cycles, expected {}, got {}", test.cycles.len(), cycles));
    }

    //
    // Verify registers
    //
    if cpu.pc != test.final_state.pc {
        errors.push(format!("Wrong PC, expected {:04X}, got {:04X}", test.final_state.pc, cpu.pc));
    }
    if cpu.a != test.final_state.a {
        errors.push(format!("Wrong A, expected {:02X}, got {:02X}", test.final_state.a, cpu.a));
    }
    if cpu.x != test.final_state.x {
        errors.push(format!("Wrong X, expected {:02X}, got {:02X}", test.final_state.x, cpu.x));
    }
    if cpu.y != test.final_state.y {
        errors.push(format!("Wrong Y, expected {:02X}, got {:02X}", test.final_state.y, cpu.y));
    }
    if cpu.s != test.final_state.s {
        errors.push(format!("Wrong S, expected {:02X}, got {:02X}", test.final_state.s, cpu.s));
    }

    //
    // Verify flags
    //
    fn check(flag: bool, shift: u8, name: String, p: u8, errors: &mut Vec<String>) {
        let expected = 1 == (p & (1 << shift)) >> shift;
        if flag != expected {
            errors.push(format!("Wrong value for flag {}, expected {}, got {}", name, expected, flag))
        }
    }


    check(cpu.p.n(), 7, "N".to_string(), test.final_state.p, &mut errors);
    check(cpu.p.v(), 6, "V".to_string(), test.final_state.p, &mut errors);
    check(cpu.p.z(), 1, "Z".to_string(), test.final_state.p, &mut errors);
    check(cpu.p.d(), 3, "D".to_string(), test.final_state.p, &mut errors);
    check(cpu.p.i(), 2, "I".to_string(), test.final_state.p, &mut errors);
    check(cpu.p.c(), 0, "C".to_string(), test.final_state.p, &mut errors);

    //
    // Verify memory
    //
    for mem_value in test.final_state.ram.iter() {
        let address = mem_value.address;
        let value = cpu.memory.get(address);
        let expected_value = mem_value.value;
        if value != expected_value {
            errors.push(format!("Wrong memory value at ${:04x}, expected {:02x}, got {:02x}",
                                mem_value.address, expected_value, value));
            // } else {
            //     println!("Correct memory: {:04X}:{:02X}", address, expected_value);
        }
    }

    // let mut result_errors: Vec<String> = Vec::new();
    let has_errors = errors.len() > 0;
    let test_name = test.name.clone(); // format!("{:02x}", test.extract_opcode());
    // result_errors.push( Errors {
    //     name: test_name.clone(),
    //     errors
    // });

    let opcode = test.extract_opcode();
    if ! has_errors {
        TestStatus::Passed(opcode, test_name.clone())
    } else {
        TestStatus::Failed(opcode, test_name.clone(), errors)
    }
}