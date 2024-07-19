use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read};
use crate::constants::ALL_DISKS;
use crate::disk::bit_stream::{BitStream, BitStreams};
use crate::disk::disk::PDisk;
use crate::disk::disk_controller::{MAX_PHASE};
use crate::disk::disk_info::DiskInfo;
use crate::disk::disk_info::DiskType::{Woz1, Woz2};
use crate::disk::dsk_to_woz::crc32;
use crate::misc::{bit, save};
use crate::ui_log;

const TMAP_SIZE: usize = MAX_PHASE;

#[derive(Clone)]
pub struct Woz {
    disk_info: DiskInfo,
    i: usize,
    pub tmap: [u8; TMAP_SIZE],
    tracks: (Option<Vec<TracksV1>>, Option<[Tracks; TMAP_SIZE]>),
    pub bit_streams: BitStreams,
    pub meta: HashMap<String, String>,
    // version contains 1 or 2, 0 if we haven't read the file yet
    pub info_chunk: InfoChunk,
}

#[derive(Clone, Default)]
struct InfoChunk {
    version: u8,
    write_protected: bool,
}

impl Default for Woz {
    fn default() -> Self {
        Self {
            disk_info: Default::default(),
            i: 0,
            tmap: [0; TMAP_SIZE],
            tracks: (None, None),
            bit_streams: Default::default(),
            meta: Default::default(),
            info_chunk: InfoChunk::default(),
        }
    }
}

impl Woz {
    // pub fn bit_streams(&self) -> BitStreams {
    //     self.bit_streams.clone()
    // }
    pub fn title(&self) -> Option<String> { self.meta.get("title").cloned() }
    pub fn version(&self) -> u8 { self.info_chunk.version }
    pub fn is_write_protected(&self) -> bool { self.info_chunk.write_protected }
}

#[derive(Default, Clone, Copy)]
struct Tracks {
    starting_block: u16,
    _block_count: u16,
    bit_count: u32,
}

const TRACK_SIZE_V1: usize = 6646;

#[derive(Clone)]
struct TracksV1 {
    file_offset: usize,
    byte_stream: Vec<u8>,  // size TRACK_SIZE_V1
    bytes_used: u16,
    bit_count: u16,
    splice_point: u16,
    splice_nibble: u8,
    splice_bit_count: u8,
}

impl PDisk for Woz {
    fn bit_streams(&self) -> &BitStreams {
        &self.bit_streams
    }

    fn bit_streams_mut(&mut self) -> &mut BitStreams {
        &mut self.bit_streams
    }

    fn save(&mut self) {
        ui_log("Woz saving {path}");
        let mut buffer: Vec<u8> = Vec::new();
        Self::woz_file_1(&mut buffer);
        Self::woz_file_2(&mut buffer, &self.bit_streams);
        let path = &self.disk_info.path;
        match save(path, &buffer) {
            Ok(_) => { ui_log(&format!("Saved {path}")); }
            Err(s) => { ui_log(&format!("Error saving {path}: {s}")); }
        }
    }

    fn disk_info(&self) -> &DiskInfo {
        &self.disk_info
    }
}

impl Woz {
    /// If `read_version` is true, then we only read the version of the file and do not
    /// parse anything else. If it's false, decode the whole file, including the bit streams
    pub fn new_with_file(filename: &str, quick: bool) -> Result<Woz, String> {
        let mut file = File::open(filename).map_err(|e| e.to_string())?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        Woz::new(&buffer, filename, quick)
    }

    fn new(bytes: &[u8], filename: &str, quick: bool) -> Result<Woz, String> {
        let mut woz = Woz::default();

        if bytes[0] != 0x32 && bytes[1] != 0x4f && bytes[2] != 0x5a && bytes[3] != 0x32 {
            return Err("Not a valid .woz file".to_string());
        }
        if bytes[4] != 0xff {
            return Err("Expected $FF".to_string());
        }

        woz.i = 8;
        let _checksum = woz.read32(bytes);
        if _checksum != 0 {
            let expected = crc32(0, &bytes[12..]);
            // println!("Checksum: {:04X} expected: {:04X}", _checksum, expected);
            // println!();
        }

        let mut end = false;
        while !end {
            let name = woz.read4_string(bytes);
            let size = woz.read32(bytes) as usize;
            if name == "INFO" {
                woz.info_chunk = woz.read_info_chunk(bytes);
                if quick {
                    end = true;
                }
            } else if name == "TMAP" {
                woz.read_tmap_chunk(bytes);
            } else if name == "TRKS" {
                woz.tracks = woz.read_tracks_chunk(bytes);
                // Skip the bitstreams, we'll decode these later
                if size == 0 {
                    panic!("Should never happen");
                }
                woz.i += size - 8 * 160;  // 160 phases, each takes 8 bytes to describe
            } else if name == "META" {
                woz.meta = woz.read_meta(bytes, size);
            } else {
                woz.skip(size);
            }
            end = woz.i >= bytes.len();
        }

        // Now decode the bitstreams if we're not just reading the version number
        if ! quick {
            let disk_type = if woz.version() == 1 { Woz1 } else { Woz2 };
            woz.disk_info = DiskInfo::new2(woz.meta.get("title").cloned(), filename,
                woz.meta.clone(), disk_type, woz.is_write_protected());
            woz.disk_info.disk_type = if woz.info_chunk.version == 1 { Woz1 } else { Woz2 };
            match woz.bytes_to_bit_streams(bytes, woz.disk_info.clone()) {
                Ok(bb) => {
                    woz.bit_streams = bb;
                    Ok(woz)
                }
                Err(err) => { Err(err) }
            }
        } else {
            woz.disk_info = DiskInfo::n(filename);
            woz.disk_info.disk_type = if woz.info_chunk.version == 1 { Woz1 } else { Woz2 };
            Ok(woz)
        }
    }

    fn push_bytes(buffer: &mut Vec<u8>, bytes: &[u8]) {
        for b in bytes { buffer.push(*b); }
    }

    pub(crate) fn push_string(buffer: &mut Vec<u8>, s: &str) {
        for c in s.chars() {
            buffer.push(c as u8);
        }
    }

    /// Encode TMAP and TRKS
    pub fn woz_file_2(buffer: &mut Vec<u8>, bit_streams: &BitStreams) {
        Woz::push_string(buffer, "TMAP");
        Woz::push_32(buffer, MAX_PHASE as u32);
        for t in &bit_streams.tmap {
            buffer.push(*t);
        }

        Woz::push_string(buffer, "TRKS");
        let chunk_size_index = buffer.len();
        Woz::push_32(buffer, 0);  // length, will be set later
        let mut block = 3;
        for bs in &bit_streams.bit_streams {
            Woz::push_16(buffer, block);
            let block_count = (bs.len() / 512 / 8) + 1;
            Woz::push_16(buffer, block_count as u16);
            Woz::push_32(buffer, bs.len() as u32);
            block += block_count as u16;
        }

        while buffer.len() != 0x600 {
            buffer.push(0);
        }

        for bs in &bit_streams.bit_streams {
            let mut bit_index = 0;
            let mut a = 0;
            while bit_index < bs.len() {
                let mut a_index = 0;
                while bit_index < bs.len() && a_index < 8 {
                    a = (a << 1) | bs.next_bit(bit_index);
                    bit_index += 1;
                    a_index += 1;
                }
                // If we reached the end of the bit stream, pad a with 0's before
                // adding it to the buffer
                while a_index < 8 {
                    a <<= 1;
                    a_index += 1;
                }
                buffer.push(a);
            }
            // Pad to the next 512 boundary
            while (buffer.len() % 512) != 0 {
                buffer.push(0);
            }
        }
        let chunk_size = (buffer.len() - chunk_size_index - 4) as u32;
        println!("Chunk size: {:08X}", chunk_size);
        Woz::set_32(buffer, chunk_size_index, chunk_size);
    }

    /// Encode INFO
    pub fn woz_file_1(buffer: &mut Vec<u8>) {
        Woz::push_string(buffer, "WOZ2");
        Woz::push_bytes(buffer, &[0xff, 0xa, 0xd, 0xa]);
        // Checksum, will set later
        Woz::push_bytes(buffer, &[0, 0, 0, 0]);

        // INFO
        Woz::push_string(buffer, "INFO");
        Woz::push_32(buffer, 60);
        buffer.push(2); // version
        buffer.push(1); // floppy
        buffer.push(1); // write protected
        buffer.push(0); // not synchronized
        buffer.push(1); // cleaned
        Woz::push_string(buffer, &format!("{: <32}", "Maple-2, by Cédric Beust"));
        buffer.push(1); // # of sides
        buffer.push(1); // boot sector format
        buffer.push(32); // ideal bit rate
        Woz::push_16(buffer, 0); // compatible with EVERYTHING
        Woz::push_16(buffer, 0); // required RAM
        Woz::push_16(buffer, 0xd); // largest track in blocks
        Woz::push_16(buffer, 0); // flux
        Woz::push_16(buffer, 0); // number of blocks for flux
        Woz::push_n(buffer, 10, 0);
    }

    pub(crate) fn push_16(buffer: &mut Vec<u8>, v: u16) {
        buffer.push((v & 0xff) as u8);
        buffer.push(((v & 0xff00) >> 8) as u8);
    }

    pub(crate) fn set_32(buffer: &mut Vec<u8>, index: usize, v: u32) {
        buffer[index] = (v & 0xff) as u8;
        buffer[index + 1] = ((v & 0xff00) >> 8) as u8;
        buffer[index + 2] = ((v & 0xff0000) >> 16) as u8;
        buffer[index + 3] = ((v & 0xff000000) >> 24) as u8;
    }

    pub(crate) fn push_32(buffer: &mut Vec<u8>, v: u32) {
        buffer.push((v & 0xff) as u8);
        buffer.push(((v & 0xff00) >> 8) as u8);
        buffer.push(((v & 0xff0000) >> 16) as u8);
        buffer.push(((v & 0xff000000) >> 24) as u8);
    }

    fn push_n(buffer: &mut Vec<u8>, count: usize, byte: u8) {
        for i in 0..count { buffer.push(byte); }
    }

    fn push_multiple(buffer: &mut Vec<u8>, s: &[u8]) {
        s.iter().for_each(|b| buffer.push(*b));
    }

    pub fn _dump(&self) {
        println!("Dumping");
        for i in 0..MAX_PHASE {
            if self.tmap[i] != 0xff {
                println!("Phase {i} -> {}", self.tmap[i]);
            }
        }

        let mut seen_tmaps: HashSet<u8> = HashSet::new();
        for i in 0..MAX_PHASE {
            let tmap = self.tmap[i];
            println!("Phase: {} tmap: {}", i, tmap);
            if seen_tmaps.contains(&tmap) || tmap == 0xff {
                continue;
            } else {
                seen_tmaps.insert(i as u8);
            }
            match &self.tracks {
                (None, Some(_trackv2)) => {
                    println!("WOZ v2");
                    // let t = trackv2[tmap as usize];
                }
                (Some(trackv1), None) => {
                    let bs = self.bit_streams();
                    let bit_stream = &bs.get_stream(i);
                    let t = &trackv1[tmap as usize];
                    let mut i = 0;
                    while i < t.bit_count {
                        let mut a = 0;
                        let mut j = 0;
                        // println!("Bit stream len: {} bit_count: {}", t.bit_stream.len(), t.bit_count);
                        while (a & 0x80) == 0 {
                            let index = (i + j) as usize;
                            if index < t.bit_count.into() {
                                a = (a << 1) | bit_stream.next_bit(index);
                                j += 1;
                            } else {
                                // truncating
                                break;
                            }
                        }
                        if a >= 0x80 {
                            print!("{:02X} ", a);
                        }
                        i += j;
                    }
                    println!();
                }
                _ => { panic!("Should have matched"); }
            }
            println!();
        }
    }

    fn bytes_to_bits(&self, bytes: &[u8], bit_count: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        let mut byte_index: usize = 0;
        while result.len() < bit_count {
            let byte = bytes[byte_index];
            for i in 0..8 {
                let bit = bit(byte, 7 - i);
                result.push(bit);
                if result.len() == bit_count { break; }
            }
            byte_index += 1;
        }
        result
    }

    fn bytes_to_bit_streams(&self, bytes: &[u8], disk_info: DiskInfo) -> Result<BitStreams, String> {
        let mut bit_streams: Vec<BitStream> = Vec::new();
        let mut seen: HashSet<u8> = HashSet::new();
        for phase in 0..TMAP_SIZE {
            let tmap = self.tmap[phase];
            if tmap != 0xff && ! seen.contains(&tmap) {
                match &self.tracks {
                    (Some(trackv1), None) => {
                        let track = &trackv1[tmap as usize];
                        let bit_slice = self.bytes_to_bits(&track.byte_stream, track.bit_count as usize);
                        bit_streams.push(BitStream::new(bit_slice));
                        seen.insert(tmap);
                    }
                    (None, Some(trackv2)) => {
                        let track = trackv2[tmap as usize];
                        let start: usize = track.starting_block as usize * 512;
                        let bit_slice =
                            self.bytes_to_bits(&bytes[start..bytes.len() - 1], track.bit_count as usize);
                        bit_streams.push(BitStream::new(bit_slice));
                        seen.insert(tmap);
                    }
                    _ => {
                        return Err("Couldn't parse this WOZ file".to_string());
                    }
                }
            }
        }

        Ok(BitStreams::new(bit_streams, self.tmap, disk_info))
    }

    fn read_meta(&mut self, bytes: &[u8], size: usize) -> HashMap<String, String> {
        let mut result = HashMap::new();
        let m = self.read_string(bytes, size);
        let strings = m.split('\n').map(|e| e.to_string()).collect::<Vec<String>>();
        for s in strings {
            let mut sp = s.split('\t');
            let key = sp.next().unwrap();
            if let Some(value) = sp.next() {
            result.insert(key.to_string(), value.to_string());
        }
        }
        result
    }

    fn read_tracks_chunk(&mut self, bytes:&[u8])
            -> (Option<Vec<TracksV1>>, Option<[Tracks; TMAP_SIZE]>) {
        let mut result1: Vec<TracksV1> = Vec::new();
        let mut result2: [Tracks; TMAP_SIZE] = [Tracks::default(); TMAP_SIZE];
        let version = self.info_chunk.version;
        if version == 1 {
            let mut max_track = 0;
            for i in 0..self.tmap.len() {
                if self.tmap[i] != 0xff && self.tmap[i] > max_track {
                    max_track = self.tmap[i];
                }
            }
            for _ in 0..=max_track {
                let start = self.i;
                let byte_stream = self.read_many(bytes, TRACK_SIZE_V1);
                let bytes_used = self.read16(bytes);
                let bit_count = self.read16(bytes);
                let splice_point = self.read16(bytes);
                let splice_nibble = self.read8(bytes);
                let splice_bit_count = self.read8(bytes);
                let _ = self.read16(bytes);  // reserved for future use
                result1.push(TracksV1 {
                    file_offset: start,
                    byte_stream, bytes_used, bit_count, splice_point, splice_nibble, splice_bit_count
                });
                // if result1.len() > 1 {
                //     assert_eq!(result1[1].bit_stream[0], 0xbf);
                //     assert_eq!(result1[1].bit_stream[1], 0x7e);
                //     assert_eq!(result1[1].bit_stream[2], 0xed);
                //     println!("{:04X} Checking phase 1: {:02X} {:02X} {:02X}", self.i,
                //         result1[1].bit_stream[0],
                //         result1[1].bit_stream[1],
                //         result1[1].bit_stream[2]);
                // }
            }
        } else {
            for i in 0..TMAP_SIZE {
                let starting_block = self.read16(bytes);
                let _block_count = self.read16(bytes);
                let bit_count = self.read32(bytes);
                result2[i] = Tracks { starting_block, _block_count, bit_count };
                // println!("Phase {}, starting_block: {} count: {}, bit_count: {}", i,
                //     starting_block, block_count, bit_count);
            }
        }
        if version == 1 {
            (Some(result1), None)
        } else {
            (None, Some(result2))
        }
    }

    fn read_tmap_chunk(&mut self, bytes: &[u8]) {
        for i in 0..TMAP_SIZE {
            self.tmap[i] = self.read8(bytes);
            // println!("TMAP[{}]={:02X}", i, self.tmap[i]);
        }
    }

    fn read_info_chunk(&mut self, bytes: &[u8]) -> InfoChunk {
        // println!("==== INFO");
        let version = self.read8(bytes);
        // println!("  Version:{}", version);
        let disk_type = if self.read8(bytes) == 1 { "5.25" } else { "3.5" };
        let write_protected = self.read8(bytes) == 1;
        // println!("  Write protected:{}",

        self.skip(57);
        InfoChunk { version, write_protected }

        // println!("  Synchronized:{}", if self.read(bytes) == 1 { "Yes" } else { "No" });
        // println!("  Cleaned:{}", if self.read(bytes) == 1 { "Yes" } else { "No" });
        // println!("  Creator:{}", self.read_string(bytes, 32));
        // println!("  Disk sides:{}", self.read(bytes));
        // println!("  Boot format: {}", match self.read(bytes) {
        //     1 => { "16 sectors" }
        //     2 => { "13 sectors" }
        //     3 => { "Both 13 and 16 sectors" }
        //     _ => { "Unknown" }
        // });
        // println!("  Optimal bit timing:{}", self.read(bytes));
        // println!("  Compatible hardware:{:0b}", self.read2(bytes));
        // println!("  Required RAM:{}K", self.read2(bytes));
        // println!("  Largest track:{}K", self.read2(bytes));
        // println!("  FLUX block:{}K", self.read2(bytes));
        // println!("  Largest FLUX track:{}K", self.read2(bytes));
        // self.skip(10);
    }

    fn skip(&mut self, n: usize) {
        self.i += n;
    }

    fn read4_string(&mut self, bytes: &[u8]) -> String {
        let mut result = String::new();
        result.push(bytes[self.i] as char);
        result.push(bytes[self.i + 1] as char);
        result.push(bytes[self.i + 2] as char);
        result.push(bytes[self.i + 3] as char);
        self.i += 4;
        result
    }

    fn read_string(&mut self, bytes: &[u8], n: usize) -> String {
        let mut result = String::new();
        for i in 0..n {
            result.push(bytes[self.i + i] as char);
        }
        self.i += n;
        result
    }

    fn read8(&mut self, bytes: &[u8]) -> u8 {
        let result = bytes[self.i];
        self.i += 1;
        result
    }

    fn read_many(&mut self, bytes: &[u8], n: usize) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for _ in 0..n {
            result.push(self.read8(bytes));
        }
        // println!("First 3 bytes: {:02X} {:02X} {:02X}", result[0], result[1], result[2]);
        result
    }

    fn read16(&mut self, bytes: &[u8]) -> u16 {
        let result = bytes[self.i] as u16 | (bytes[self.i + 1] as u16) << 8;
        self.i += 2;
        result
    }

    fn read32(&mut self, bytes: &[u8]) -> u32 {
        let result: u32 = (bytes[self.i] as u32)
            | (bytes[self.i + 1] as u32) << 8
            | (bytes[self.i + 2] as u32) << 16
            | (bytes[self.i + 3] as u32) << 24;
        self.i += 4;
        result
    }

}

pub fn _test_wozv1_2() {
    let mut file = File::open(ALL_DISKS[22].path()).unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let woz = Woz::new_with_file(&ALL_DISKS[22].path(), false).unwrap();
    // Offsets of the tracks in the file, incremented by 1a00 for each track

    for index in 0..100 {
        let a = woz.bit_streams.bit_streams[4].next_byte(index * 8);
        let expected = buffer[index + 0x1b00];
        assert_eq!(a, expected, "Index {} expected {:02X} but got {:02X}", index, expected, a);
        println!("Success {:02X} == {:02X}", a, expected);
    }
}

pub fn _test_wozv1_1() {
    let mut file = File::open(ALL_DISKS[22].path()).unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut expected: HashMap<usize, [u8; 3]> = HashMap::new();
    expected.insert(0, [0xFF, 0x7F, 0xBF]); // 100
    expected.insert(3, [0xBF, 0x7E, 0xED]); // 1b00
    expected.insert(7, [0, 0, 0]); // 3500
    expected.insert(11, [0xB7, 0xF7, 0xBF]);  // 4f00
    expected.insert(16, [0xdf, 0x7f, 0x7e]);  // 6900
    // expected.insert(19, []);  // 8300
    // 9D00 Parsing track v1, phase: 23 tmap: 6
    // B700 Parsing track v1, phase: 27 tmap: 7
    // D100 Parsing track v1, phase: 31 tmap: 8
    // EB00 Parsing track v1, phase: 36 tmap: 9
    // 10500 Parsing track v1, phase: 39 tmap: 10
    // 11F00 Parsing track v1, phase: 43 tmap: 11
    // 13900 Parsing track v1, phase: 47 tmap: 12
    // 15300 Parsing track v1, phase: 51 tmap: 13
    // 16D00 Parsing track v1, phase: 55 tmap: 14
    // 18700 Parsing track v1, phase: 59 tmap: 15

    let woz = Woz::new_with_file(&ALL_DISKS[22].path(), false);
    match woz {
        Ok(woz) => {
            let bs = woz.bit_streams();
            let s = bs.get_stream(0);
            let mut i = 0;
            let mut line: Vec<u8> = Vec::new();
            let mut ln = 0;
            while i < s.len() - 30 {
                let (size, nibble) = s.next_nibble(i);
                i += size;
                line.push(nibble);
                if line.len() == 16 {
                    print!("{:04X}: ", ln);
                    ln += 16;
                    for b in &line {
                        print!("{:02X} ", b);
                    }
                    println!();
                    line.clear();
                }
                if nibble == 0xdb || nibble == 0xab || nibble == 0xbf {
                    // println!("Nibble: {:02X} (size: {size})", nibble);
                }
            }
            for phase in 0..MAX_PHASE {
                match expected.get(&phase) {
                    None => {}
                    Some(e) => {
                        let bs = woz.bit_streams();
                        let s = bs.get_stream(phase);
                        assert_eq!(s.next_byte(0), e[0]);
                        assert_eq!(s.next_byte(8), e[1]);
                        assert_eq!(s.next_byte(16), e[2]);
                        println!("Phase {phase} passed the test");
                        // println!("Phase {phase}: {:02X} {:02X} {:02X}",
                        //     *stream.get(0).unwrap(), stream.get(1).unwrap(), stream.get(1).unwrap()
                        // )
                    }
                }
            }
        }
        Err(_) => {
            assert!(false, "Couldn't read file");
        }
    }
}