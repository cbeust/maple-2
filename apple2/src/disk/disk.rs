use crossbeam::channel::Sender;
use dyn_clone::DynClone;
use crate::disk::bit_stream::{AnalyzedTrack, BitStreams};
use crate::disk::disk_info::DiskInfo;
use crate::disk::dsk::Dsk;
use crate::disk::woz::Woz;
use crate::messages::ToUi;

/// Each disk format (.dsk, .woz) supplies its owm implementation of this trait
pub trait PDisk: DynClone {
    fn bit_streams(&self) -> &BitStreams;
    fn bit_streams_mut(&mut self) -> &mut BitStreams;
    fn save(&mut self);
    fn disk_info(&self) -> &DiskInfo;
}

pub(crate) struct Disk {
    // Bit streams for each phase (0..MAX_PHASE)
    pdisk: Box<dyn PDisk>,

    // pub(crate) bit_streams: BitStreams,
    // 0 or 1
    // pub(crate) drive_number: usize,
    pub(crate) bit_position: usize,
    sender: Option<Sender<ToUi>>,
}

impl Clone for Disk {
    fn clone(&self) -> Self {
        Self {
            pdisk: dyn_clone::clone_box(&*self.pdisk),
            bit_position: self.bit_position,
            sender: self.sender.clone(),
        }
    }
}

//     fn clone_from(&mut self, source: &Self) {
//         todo!()
//     }
// }

impl Disk {
    pub fn new(path: &str, quick: bool, sender: Option<Sender<ToUi>>) -> Result<Disk, String> {
        let pdisk = Self::new_pdisk(path, quick)?;
        Ok(Self {
            pdisk,
            bit_position: 0,
            sender,
        })
    }

    pub fn disk_info(&self) -> DiskInfo {
        self.pdisk.disk_info().clone()
    }

    pub fn set_bit_and_advance(&mut self, phase_160: usize, bit: u8) {
        self.pdisk.bit_streams_mut().set_bit_and_advance(phase_160, self.bit_position, bit);
        let len = self.pdisk.bit_streams().get_stream(phase_160).len();
        // let stream = &mut streams.get_stream_mut(phase_160);
        // stream.set_bit(self.bit_position, bit);
        self.bit_position = (self.bit_position + 1) % len;
    }

    fn new_pdisk(path: &str, quick: bool) -> Result<Box<dyn PDisk>, String> {
        if path.to_lowercase().ends_with(".woz") {
            match Woz::new_with_file(path, quick) {
                Ok(p) => { Ok(Box::new(p)) }
                Err(s) => { Err(s) }
            }
        } else if path.to_lowercase().ends_with(".dsk") {
            match Dsk::new_with_file(path, quick) {
                Ok(p) => { Ok(Box::new(p) ) }
                Err(s) => { Err(s) }
            }
        } else {
            panic!("Unknown disk format");
        }
    }

    pub fn get_stream_len(&self, phase_160: usize) -> usize {
        self.pdisk.bit_streams().get_stream(phase_160).len()
    }

    pub fn analyze_track(&self, phase_160: usize) -> AnalyzedTrack {
        self.pdisk.bit_streams().get_stream(phase_160).analyze_track()
    }

    pub(crate) fn save(&mut self) {
        println!("Ready to write tracks");
        self.pdisk.save();
        // let path = self.disk_info.path.to_lowercase();
        // if path.ends_with(".woz") {
        //     Woz::save(&self.disk_info.path, &self.bit_streams);
        // } else if path.ends_with(".dsk") {
        //     Dsk::save(&self.disk_info.path, &self.bit_streams);
        // } else {
        //     ui_log(&format!("Don't know how to save to {}", self.disk_info.path));
        // }
        // // write_info.display();
        println!("... done writing tracks");
    }

    pub(crate) fn peek_bit(&mut self, bit_position: usize, phase160: usize) -> u8 {
        let streams = self.pdisk.bit_streams();
        let stream = streams.get_stream(phase160);
        stream.next_bit(bit_position)
    }

    #[allow(unused)]
    pub(crate) fn peek_nibbles(&mut self, count: usize, phase160: usize) -> (usize, Vec<u8>) {
        let mut result: Vec<u8> = Vec::new();
        let mut position = self.bit_position;
        let len = self.pdisk.bit_streams().get_stream(phase160).len();
        let mut bits_seen = 0;
        for _ in 0..count {
            let mut a = 0;
            while (a & 0x80) == 0 {
                a = (a << 1) | self.peek_bit(position, phase160);
                bits_seen += 1;
                position = (position + 1) % len;
            }
            result.push(a);
        }

        (bits_seen, result)
    }

    pub(crate) fn next_bit(&mut self, phase160: usize) -> u8 {
        // if self.peek_nibbles(1).1 == [0xf0] {
        //     println!("BREAK");
        // }
        let result = self.peek_bit(self.bit_position, phase160);
        // let stream = self.bit_streams.get_stream(phase);
        let len = self.pdisk.bit_streams().get_stream(phase160).len();
        self.bit_position = (self.bit_position + 1) % len;
        // let position = self.bit_position;
        // while self.peek_bit(self.bit_position) == 0 {
        //     self.bit_position = (self.bit_position + 1) % len;
        // }
        // if len > 0 && self.bit_position < len- 1 {
        //     self.bit_position += 1;
        // } else {
        //     println!("Wrapping the stream at position: {}, len: {}", self.bit_position, len);
        //     self.bit_position = 0;
        // }
        result
    }
}