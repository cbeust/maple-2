use std::fs::File;
use std::io::Read;
use std::ops::BitXor;
use crate::debug::hex_dump;
use crate::disk::bit_stream::{AreaType, BitStream, BitStreams};
use crate::disk::disk::PDisk;
use crate::disk::disk_controller::*;
use crate::disk::disk_info::{DiskInfo, WozVersion};
use crate::ui_log;

#[derive(Clone)]
pub struct Dsk {
    disk_info: DiskInfo,
    bit_streams: BitStreams,
}

impl PDisk for Dsk {
    fn bit_streams(&self) -> &BitStreams {
        &self.bit_streams
    }

    fn bit_streams_mut(&mut self) -> &mut BitStreams {
        &mut self.bit_streams
    }

    fn save(&mut self) {
        todo!()
    }

    fn disk_info(&self) -> &DiskInfo {
        &self.disk_info
    }
}

impl Dsk {
    pub fn new_with_file(filename: &str, quick: bool) -> Result<Dsk, String> {
        if quick {
            Ok(Dsk {
                disk_info: DiskInfo::n(filename),
                bit_streams: Default::default(),
            })
        } else {
            let mut file = File::open(filename).map_err(|e| e.to_string())?;
            let mut buffer: Vec<u8> = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            Dsk::bytes_to_bit_streams(filename, &buffer)
        }
    }

    /// For the .dsk format, we use the same TMAP in default woz disks:
    /// [0, 0, 0xff, 1, 1, 1, 0xff, 2, 2, 2, ...]
    /// result[0..3] = buffer for track 0
    /// result[4..7] = buffer for track 1
    /// ...
    fn bytes_to_bit_streams(filename: &str, bytes: &[u8]) -> Result<Dsk, String> {
        let mut tracks: Vec<BitStream> = Vec::new();
        let mut track = 0;

        // Fill the tracks first: 35 valid tracks, the last 5 are random bits
        while track < MAX_PHASE / 4 - 5 { // && index < bytes.len() {
            let start = track * TRACK_SIZE_BYTES;
            let end = /* min(bytes.len(), */ start + TRACK_SIZE_BYTES;
            if start < bytes.len() && end <= bytes.len() {
                let slice = &bytes[start..end];
                tracks.push(BitStream::new(Dsk::encode_track(slice, track as u8)));
            }
            track += 1;
        };

        // Fill the tmap
        let mut tmap: [u8; MAX_PHASE] = [0xff; MAX_PHASE];
        tmap[0] = 0;   // 0.0
        tmap[1] = 0;   // 0.25
        // tmap[2] already set to 0xff
        let mut track = 1;
        // Wripte a TMAP for the tracks 0..35, the last 5 are random
        for phase in (4..MAX_PHASE - 20).step_by(4) {
            tmap[phase - 1] = track;
            tmap[phase] = track;
            if phase + 1 < MAX_PHASE - 1 { tmap[phase + 1] = track; }
            track += 1;
        }

        // Now fill the bit_streams according to the TMAP
        let mut bit_streams: Vec<BitStream> = Vec::new();
        for phase in 0..tmap.len() {
            let t = tmap[phase];
            let bs = if t != 0xff {
                tracks[t as usize].clone()
            } else {
                BitStream::random()
            };
            bit_streams.push(bs);
        }

        let mut dsk = Dsk {
            disk_info: DiskInfo::n(filename),
            bit_streams: BitStreams::new(bit_streams, tmap, DiskInfo::default()),
        };
        dsk.disk_info.woz_version = WozVersion::Dsk;
        Ok(dsk)
    }

    pub fn decode_4_and_4(a: u8, b: u8) -> u8 {
        ((a << 1) | 0x55) & (b | 0xaa)
    }

    pub(crate) fn encode_6_and_2(values: &[u8]) -> Vec<u8> {
        let mut result: [u8; DATA_FIELD_SIZE] = [0; DATA_FIELD_SIZE];
        let bit_reverse = [0, 2, 1, 3];
        for c in 0..84 {
            let b0 = bit_reverse[values[c] as usize & 3_usize];
            let b1 = bit_reverse[values[c + 86] as usize & 3_usize] << 2;
            let b2 = bit_reverse[values[c + 172] as usize & 3_usize] << 4;
            result[c] = b0 | b1 | b2;
        }
        result[84] = bit_reverse[(values[84_usize] & 3) as usize] << 0 |
            bit_reverse[(values[170_usize] & 3) as usize] << 2;
        result[85] = bit_reverse[(values[85_usize] & 3) as usize] << 0 |
            bit_reverse[(values[171_usize] & 3) as usize] << 2;

        //         repeat(256) { c ->
        //             result[86 + c] = src[c].and(0xff) shr 2
        //         }
        for c in 0..256 {
            result[86_usize + c as usize] = (values[c as usize] & 0xff) >> 2;
        }

        // Exclusive OR each byte with the one before it.
        result[342] = result[341];
        let mut location = 342;
        while location > 1 {
            location -= 1;
            result[location] = result[location].bitxor(result[location - 1]);
        }
        // Map six-bit values up to full bytes
        for c in 0..=342 {
            result[c] = WRITE_TABLE[result[c] as usize]
        }
        result.to_vec()
    }

    fn write_4and4(bytes: &mut Vec<u8>, value: u8) {
        Dsk::write8(bytes, vec![(value >> 1) | 0xaa_u8, value | 0xaa]);
    }

    pub(crate) fn write_sync(bytes: &mut Vec<u8>, count: u8) {
        for _ in 0..count {
            Dsk::write8(bytes, vec![0xff]);
            Dsk::write1(bytes, vec![0, 0]);
        }
    }

    pub(crate) fn write8(bytes: &mut Vec<u8>, values: Vec<u8>) {
        for value in values {
            for it in 0..8 {
                let shift = 7 - it;
                let mask = 1 << shift;
                let bit = (value & mask) >> shift;
                bytes.push(bit);
            }
        }
    }

    fn write1(bytes: &mut Vec<u8>, values: Vec<u8>) {
        for value in values {
            if value == 0 || value == 1 {
                bytes.push(value);
            } else {
                panic!("Illegal bit: {}", value);
            }
        }
    }

    pub fn encode_track(bytes: &[u8], track: u8) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        Dsk::write_sync(&mut result, 16);
        for sector in 0..16 {
            Dsk::write8(&mut result, vec![0xd5, 0xaa, 0x96]);
            Dsk::write_4and4(&mut result, 0xfe);
            Dsk::write_4and4(&mut result, track);
            Dsk::write_4and4(&mut result, sector);
            Dsk::write_4and4(&mut result, 0xfe.bitxor(track).bitxor(sector));
            Dsk::write8(&mut result, vec![0xde, 0xaa, 0xeb]);
            Dsk::write_sync(&mut result, 7);

            Dsk::write8(&mut result, vec![0xd5, 0xaa, 0xad]);
            let start = LOGICAL_SECTORS[sector as usize] as usize * SECTOR_SIZE_BYTES;
            let end = start + SECTOR_SIZE_BYTES; // std::cmp::max(bytes.len(), start + 256_usize);
            // println!("Encoding track {} sector {} (logical {}) at {}-{}", track, sector,
            //          logical_sector, start, end);
            let encoded = Dsk::encode_6_and_2(&bytes[start..end]);
            // println!("Encoding track {} sector {} logical {} at {}..{}, total bytes: {}",
            //          track, sector, logical_sector, start, end, encoded.len());
            Dsk::write8(&mut result, encoded);
            Dsk::write8(&mut result, vec![0xde, 0xaa, 0xeb]);
            Dsk::write_sync(&mut result, 16);
        }

        // Add the track suffix
        // let track_position = result.len();
        // let tp = track_position + 7;
        // Dsk::write8(&mut result, vec![
        //     (tp >> 3) as u8,
        //     (tp >> 11) as u8,
        //     track_position as u8,
        //     (track_position >> 8) as u8,
        //     0, 0, 0xff, 10
        // ]);

        result
    }

    /// Decode an array of 343 nibbles into 256 bytes
    pub fn decode_6_and_2(source: &[u8]) -> [u8; 256] {
        assert_eq!(source.len(), DATA_FIELD_SIZE);
        let mut read_table: [u8; 256] = [0; 256];
        for i in 0..WRITE_TABLE.len() {
            read_table[WRITE_TABLE[i] as usize] = (0xff & i) as u8;
        }
        let mut temp: [u8; 342] = [0; 342];
        let mut current = 0;
        let mut last = 0;
        let mut i = temp.len() - 1;
        while i > 255 {
            let t = read_table[source[current] as usize];
            temp[i] = t ^ last;
            last ^= t;
            current += 1;
            i -= 1;
        }
        for i in 0..256 {
            let t = read_table[source[current] as usize];
            temp[i] = t ^ last;
            last ^= t;
            current += 1;
        }

        let mut result: [u8; 256] = [0; 256];
        let mut p = temp.len() - 1;
        for i in 0..256 {
            let mut a = temp[i] << 2;
            a += ((temp[p] & 1) << 1) + ((temp[p] & 2) >> 1);
            result[i] = a;
            temp[p] >>= 2;
            p -= 1;
            if p < 256 {
                p = temp.len() - 1;
            }
        }

        // hex_dump(&result);
        result
    }

    pub fn save(path: &str, bit_streams: &BitStreams) {
        ui_log("Dsk saving {path}");
        let mut buffer: [u8; DSK_SIZE_BYTES] = [0; DSK_SIZE_BYTES]; // 40 tracks, 16 sectors per track
        let mut track = 0;
        let mut sector = 0;
        for t in 0..MAX_TRACK_DSK {
            let bs = bit_streams.get_stream(t * 4);
            let nibbles = bs.analyze_track().nibbles;
            let mut i = 0;
            while i < nibbles.len() {
                let n = nibbles[i];
                match n.area_type {
                    AreaType::AddressPrologue => {}
                    AreaType::AddressContent => {
                        track = Self::decode_4_and_4(nibbles[i + 2].value, nibbles[i + 3].value);
                        sector = Self::decode_4_and_4(nibbles[i + 4].value, nibbles[i + 5].value);
                        i += 10;
                    }
                    AreaType::DataContent => {
                        let values: Vec<u8> =
                            nibbles[i..i + DATA_FIELD_SIZE].iter().map(|n| n.value).collect();
                        let bytes = Dsk::decode_6_and_2(&values);
                        let start = LOGICAL_SECTORS[sector as usize] as usize;
                        let offset = (track as usize * 16 + start) * SECTOR_SIZE_BYTES;
                        buffer[offset..offset + SECTOR_SIZE_BYTES].clone_from_slice(&bytes);
                        hex_dump(&bytes);
                        println!("Storing {track}/{sector} at {offset:02X}");
                        i += DATA_FIELD_SIZE;
                    }
                    _ => {}
                }
                i += 1;
            }
        }

        let path = "d:\\Apple Disks\\saved.dsk";

        match crate::misc::save(path, &buffer) {
            Ok(_) => { ui_log(&format!("Saved {path}")); }
            Err(s) => { ui_log(&format!("Error saving {path}: {s}")); }
        }
    }
}

#[allow(non_snake_case)]
pub const WRITE_TABLE: [u8; 64] = [
    0x96, 0x97, 0x9a, 0x9b, 0x9d, 0x9e, 0x9f, 0xa6,
    0xa7, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb2, 0xb3,
    0xb4, 0xb5, 0xb6, 0xb7, 0xb9, 0xba, 0xbb, 0xbc,
    0xbd, 0xbe, 0xbf, 0xcb, 0xcd, 0xce, 0xcf, 0xd3,
    0xd6, 0xd7, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde,
    0xdf, 0xe5, 0xe6, 0xe7, 0xe9, 0xea, 0xeb, 0xec,
    0xed, 0xee, 0xef, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6,
    0xf7, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];
