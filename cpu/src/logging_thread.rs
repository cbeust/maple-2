use std::time::SystemTime;
use crossbeam::channel::{Receiver, RecvError, Sender};
use crate::config::Config;
use crate::constants::OPERANDS_6502;
use crate::disassembly::{Disassemble, RunDisassemblyLine};
use crate::messages::{LogMsg, ToCpuUi, ToLogging};

pub struct Logging {
    pub receiver: Receiver<ToLogging>,
    sender: Option<Sender<ToCpuUi>>,
    pub config: Config,
    active: bool,
    last_received_message: SystemTime,
}

impl Logging {
    pub fn new(config: Config, receiver: Receiver<ToLogging>, sender: Option<Sender<ToCpuUi>>)
            -> Self {
        Self {
            receiver, config,
            sender,
            active: false,
            last_received_message: SystemTime::now(),
        }
    }

    pub fn run(&mut self) {
        let mut run = true;
        while run {
            if let Ok(message) = self.receiver.try_recv() {
                self.last_received_message = SystemTime::now();
                if ! self.active {
                    if let Some(sender) = &self.sender {
                        sender.send(ToCpuUi::LogStarted).unwrap();
                    }
                }
                self.active = true;
                match message {
                    ToLogging::Log(log_msg) => {
                        self.log(log_msg);
                    }
                    ToLogging::End => {
                        run = true;
                    }
                }
            }
            if self.active {
                if let Ok(t) = self.last_received_message.elapsed() {
                    if t.as_millis() > 10 {
                        if let Some(sender) = &self.sender {
                            sender.send(ToCpuUi::LogEnded).unwrap();
                        }
                        self.active = false;
                    }
                }
            }
        }
    }

    fn log(&self, LogMsg { global_cycles, pc, operand, byte1, byte2,
            resolved_address, resolved_value, resolved_read, a, x, y, p, s }: LogMsg) {

        let operands = OPERANDS_6502;

        let disassembly_line = Disassemble::disassemble2(&operands, pc,
            &operand, byte1, byte2);

        let d = RunDisassemblyLine::new(global_cycles, disassembly_line,
            resolved_address, resolved_value, resolved_read, operand.size,
            a, x, y, p, s);
        let stack: Vec<u16> = Vec::new(); // self.format_stack();
        // println!("{} {} {}", d.to_asm(), self.p, stack);
        // println!("{}", d.to_csv());

        if self.config.trace_to_file && self.config.csv {
            log::info!("{}", d.to_csv());
        } else {
            log::info!("{} {} {:?}", d.to_asm(), p, stack);
        }
    }
}