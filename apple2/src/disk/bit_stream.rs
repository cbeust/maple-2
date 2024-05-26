use std::fmt::{Debug, Display, Formatter};
use rand::random;
use crate::debug::{hex_dump_fn};
use crate::disk::bit_stream::AreaType::Unknown;
use crate::disk::disk_controller::{MAX_PHASE};
use crate::disk::disk_info::DiskInfo;
use crate::misc::bit;

/// A stream of bit that contains the content of a track
#[derive(Default, Clone)]
pub struct BitStream {
    bits: Vec<u8>,
    pub(crate) random: bool,
}

#[derive(Clone, Copy)]
pub struct Nibble {
    pub value: u8,
    /// Number of sync bits following that nibble
    pub sync_bits: u16,
    pub area_type: AreaType,
}

impl Nibble {
    pub(crate) fn new(value: u8, sync_bits: u16) -> Self {
        Self { value, sync_bits, area_type: AreaType::Unknown }
    }

    fn is(&self, value: u8) -> bool { self.value == value }

    fn area(&self, area_type: AreaType) -> Nibble {
        Nibble{ value: self.value, sync_bits: self.sync_bits, area_type: area_type }
    }

    pub(crate) fn to_bits(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for i in 0..8 {
            result.push(bit(self.value, 7 - i));
        }
        if self.sync_bits >= 2 {
            for _ in 0..self.sync_bits { result.push(0); }
        }

        result
    }
}

impl Display for Nibble {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // let sync_bits = if self.sync_bits >= 2 {
        //     format!("({:-2})", self.sync_bits)
        // } else {
        //     "    ".to_string()
        // };
        // f.write_str(&format!("{:02X}{}", self.value, sync_bits)).unwrap();
        let sync_bits = if self.sync_bits > 0 {
            format!("^{:02}", self.sync_bits)
        } else {
            "   ".to_string()
        };
        f.write_str(&format!("{:02X}{}", self.value, sync_bits)).unwrap();
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AreaType {
    AddressPrologue, AddressContent, AddressEpilogue,
    DataPrologue, DataContent, DataEpilogue,
    Unknown,
}

#[derive(PartialEq)]
pub enum TrackType {
    Standard,
    Nonstandard, Empty
}

pub struct AnalyzedTrack {
    pub(crate) nibbles: Vec<Nibble>,
    pub(crate) track_type: TrackType,
}

impl BitStream {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            bits: buffer, random: false,
        }
    }

    pub fn random() -> Self {
        Self {
            bits: vec![0], random: true
        }
    }

    /// Analyze the track to find out if it's standard, nonstandard, or empty
    ///
    /// TODO: Do more checks, e.g.
    /// - data field size is 343
    /// - verify checksums
    /// - maybe try to guess the markers based on 4-4 detection
    pub fn analyze_track(&self) -> AnalyzedTrack {
        // How many valid D5 AA 96 ... DE AA we found
        let mut address_prologues_found = 0;
        let mut address_epilogues_found = 0;
        // How many valid D5 AA AD ... DE AA we found
        let mut data_prologues_found = 0;
        let mut data_epilogues_found = 0;

        let nibbles = &self.find_nibble_areas();
        let track_type = if self.random {
            TrackType::Empty
        } else {
            for nibble in nibbles {
                use AreaType::*;
                match nibble.area_type {
                    AddressPrologue if nibble.value == 0xd5 => { address_prologues_found += 1; }
                    AddressContent => {}
                    AddressEpilogue => if nibble.value == 0xde { address_epilogues_found += 1;},
                    DataPrologue => if nibble.value == 0xd5 { data_prologues_found += 1; }
                    DataContent => {}
                    DataEpilogue => if nibble.value == 0xde { data_epilogues_found += 1; }
                    Unknown => {}
                    _ => {}
                }
            }

            if address_prologues_found == 16 && address_epilogues_found == 16 &&
                  data_prologues_found == 16 && data_epilogues_found == 16 {
                TrackType::Standard
            } else {
                TrackType::Nonstandard
            }
        };

        AnalyzedTrack {
            nibbles: nibbles.to_vec(),
            track_type,
        }
    }

    pub fn to_nibbles(&self) -> Vec<Nibble> {
        let mut i = 0;
        let len = self.bits.len();
        let mut result: Vec<Nibble> = Vec::new();
        while i < len {
            // while i < len && self.bits[i] == 0 { i += 1; }
            let mut value = 0;
            while i < len && (value & 0x80) == 0 {
                value = (value << 1) | self.bits[i];
                i += 1;
            }
            let mut sync_bits: u16 = 0;
            if i < len && self.bits[i] == 0 && self.bits[(i + 1) % self.len()] == 0 {
                while i < len && self.bits[i] == 0 {
                    sync_bits += 1;
                    i += 1;
                }
            }
            result.push(Nibble { value, sync_bits, area_type: Unknown });
        }

        result
    }

    /// Analyze the nibbles and assign them areas if possible
    pub fn find_nibble_areas(&self) -> Vec<Nibble> {
        let nibbles = self.to_nibbles();

        let mut result: Vec<Nibble> = Vec::new();
        if nibbles.len() < 3 {
            return result;
        }

        use AreaType::*;

        let mut i = 0;
        let mut current_nibbles: Vec<Nibble> = Vec::new();
        let mut current_area_type = Unknown;
        let mut current_area_start: usize = 0;

        while i < nibbles.len() {
            if i < nibbles.len() - 3 && nibbles[i].is(0xd5) && nibbles[i+1].is(0xaa) && nibbles[i+2].is(0x96) {
                for n in &current_nibbles {
                    result.push(n.area(current_area_type));
                }
                current_nibbles = Vec::new();

                for j in 0..3 {
                    result.push(nibbles[i + j].area(AddressPrologue));
                }

                i += 3;
                current_area_start = i;
                current_area_type = AddressContent;
            } else if i < nibbles.len() - 3 && nibbles[i].is(0xd5) && nibbles[i+1].is(0xaa) && nibbles[i+2].is(0xad) {
                for n in &current_nibbles {
                    result.push(n.area(current_area_type));
                }
                current_nibbles = Vec::new();

                for j in 0..3 {
                    result.push(nibbles[i + j].area(DataPrologue));
                }
                i += 3;
                current_area_type = DataContent;
            } else if i < nibbles.len() - 2 && nibbles[i].is(0xde) && nibbles[i+1].is(0xaa) {
                let this_area_type = if current_area_type == AddressContent { AddressEpilogue } else { DataEpilogue };
                for n in &current_nibbles {
                    result.push(n.area(current_area_type));
                }
                current_nibbles = Vec::new();

                for j in 0..2 {
                    result.push(nibbles[i + j].area(this_area_type));
                }
                i += 2;
                current_area_start = i;
                current_area_type = Unknown;
            } else {
                current_nibbles.push(nibbles[i]);
                i += 1;
            }
        }
        for n in &current_nibbles {
            result.push(n.area(current_area_type));
        }

        // let mut i = 0;
        // for nibble in &result {
        //     print!("{} ", nibble);
        //     i += 1;
        //     if i == 16 {
        //         println!("");
        //         i = 0;
        //     }
        // }
        result
    }

    pub fn copy(&self) -> Self {
        Self { bits: self.bits.clone(), random: self.random }
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn next_bit(&self, bit_index: usize) -> u8 {
        if self.random {
            if random::<f32>() < 0.3 { 1 } else { 0 }
        } else {
            self.bits[bit_index]
        }
    }

    pub(crate) fn set_bit(&mut self, bit_index: usize, value: u8) {
        self.bits[bit_index] = value;
    }


    /// Return the next byte
    pub fn next_byte(&self, bit_index: usize) -> u8 {
        let mut result = 0;
        let mut address = bit_index;
        for _ in 0..8 {
            result = (result << 1) | self.next_bit(address);
            address = if address + 1 >= self.len() {
                0
            } else {
                address + 1
            }
        }
        result
    }

    /// Return the next nibble (>= 0x80) and the number of bits we had to consume to get there
    /// (number_of_bits, nibble)
    pub fn next_nibble(&self, bit_index: usize) -> (usize, u8) {
        let mut result = 0;
        let mut count = 0;
        let mut address = bit_index + count;
        while (result & 0x80) != 0x80 {
            result = (result << 1) | self.next_bit(address);
            count += 1;
            address = if bit_index + count >= self.len() {
                0
            } else {
                bit_index + count
            }
        }
        (count, result)
    }
}

/// The content of an entire disk, with 160 bit streams
#[derive(Clone)]
pub struct BitStreams {
    pub bit_streams: Vec<BitStream>,
    pub tmap: [u8; MAX_PHASE],
    pub disk_info: DiskInfo,
    random: BitStream,
}

impl BitStreams {
    pub fn copy(&self) -> Self {
        Self { bit_streams: self.bit_streams.clone(), tmap: self.tmap,
            disk_info: self.disk_info.clone(), random: BitStream::random() }
    }

    pub fn get_stream(&self, phase: usize) -> &BitStream {
        assert!((0..MAX_PHASE).contains(&(phase)));
        let t = self.tmap[phase];
        if t != 0xff {
            if self.bit_streams.len() == 0 {
                println!("BUG ZERO SIZE STREAM");
            }
            &self.bit_streams[t as usize]
        } else {
            &self.random
        }
    }

    pub fn set_bit_and_advance(&mut self, phase_160: usize, bit_position: usize, bit: u8) {
        self.bit_streams[self.tmap[phase_160] as usize].set_bit(bit_position, bit);
    }

    pub fn get_stream_mut(&mut self, phase: usize) -> &mut BitStream {
        assert!((0..MAX_PHASE).contains(&(phase)));
        let t = self.tmap[phase];
        if t != 0xff {
            &mut self.bit_streams[t as usize]
        } else {
            &mut self.random
        }
    }

    pub fn len(&self, phase: usize) -> usize {
        self.get_stream(phase).len()
    }

    pub(crate) fn dump(&self, phase_160: u8) {
        println!("Dumping phase {phase_160}");
        let stream = &self.get_stream(phase_160 as usize);
        let nibbles = stream.to_nibbles();
        hex_dump_fn(&nibbles, |n| format!("{}", n));
    }

}

impl Default for BitStreams {
    fn default() -> Self {
        Self {
            bit_streams: Vec::new(),
            tmap: [0; MAX_PHASE],
            disk_info: DiskInfo::default(),
            random: BitStream::random(),
        }
    }
}

impl BitStreams {
    pub(crate) fn new(bit_streams: Vec<BitStream>, tmap: [u8; MAX_PHASE], disk_info: DiskInfo)
            -> BitStreams {
        Self {
            bit_streams, tmap, disk_info, random: BitStream::random(),
        }
    }
}