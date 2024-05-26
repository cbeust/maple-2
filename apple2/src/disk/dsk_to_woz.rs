use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::debug::{hex_dump, hex_dump_at};
use crate::disk::dsk::{Dsk, WRITE_TABLE};
use crate::disk::woz::Woz;

pub fn woz_to_dsk(path: &str) -> Result<String, String> {
    let woz = Woz::new_with_file(path, false)?;
    println!("Bit stream count: {}", woz.bit_streams.bit_streams.len());

    for phase in (0..160).step_by(4) {
        let mut buffer: Vec<u8> = Vec::new();
        let bs = &woz.bit_streams.bit_streams[phase];
        let mut index = 0;
        while index < bs.len() {
            let (increment, nibble) = bs.next_nibble(index);
            buffer.push(nibble);
            index += increment;
        }
        // println!("Read track {}", phase / 4);
        // for i in 0..30 {
        //     print!("{:02X} ", buffer[i]);
        // }
        decode_track(&buffer);
        // println!();
    }
    // Save the file
    let stem = Path::new(path).file_stem().unwrap().to_str().unwrap();
    let file_out = format!("{}.woz", stem);
    // let mut file = File::create(&file_out).map_err(|e| e.to_string())?;
    // file.write(&buffer).map(|_| file_out).map_err(|e| e.to_string())

    Ok(file_out)
}

struct Sector {
    sector_number: u8,
    bytes: [u8; 256],
}

/// track contains a nibble track (FF FF D5 AA 96...)
/// Return 16 tracks of 256 bytes, in the correct sector order
fn decode_track(track: &[u8]) {
    let mut result: Vec<u8> = Vec::new();
    let mut i = 0;

    let mut tracks: Vec<Sector> = Vec::new();

    while i < track.len() {
        let mut sector_number = 0xff;
        let mut track_number = 0xff;
        if track.len() > 10 {
            if track[i] == 0xd5 && track[i + 1] == 0xaa && track[i + 2] == 0x96 {
                // hex_dump_at(track, i as u16, 48);
                let volume = Dsk::decode_4_and_4(track[i + 3], track[i + 4]);
                i += 5; // skip volume
                track_number = Dsk::decode_4_and_4(track[i], track[i + 1]);
                sector_number = Dsk::decode_4_and_4(track[i + 2], track[i + 3]);
                let checksum = Dsk::decode_4_and_4(track[i + 4], track[i + 5]);
                if volume ^ track_number ^ sector_number != checksum {
                    panic!("Checksums do not match!");
                }
                // println!();
                i += 10;
            }

            if track[i] == 0xd5 && track[i + 1] == 0xaa && track[i + 2] == 0xad {
                let bytes = Dsk::decode_6_and_2(&track[i + 3..i + 346]);
                tracks.push(Sector { sector_number, bytes });
                assert_eq!(track[i + 346], 0xde);
                assert_eq!(track[i + 347], 0xaa);
            }
        }
        i += 1;
    }
}

pub fn dsk_to_woz(path: &str) -> Result<String, String> {
    let mut buffer: Vec<u8> = Vec::new();

    // Woz::woz_file_start(&mut buffer);


    // TRKS
    let mut dsk_buffer: Vec<u8> = Vec::new();
    let mut dsk = File::open(path).map_err(|e| e.to_string())?;
    dsk.read_to_end(&mut dsk_buffer).map_err(|e| e.to_string())?;

    Woz::push_string(&mut buffer, "TRKS");
    // Memorize the index where we will store the length once we know it
    let chunk_size_index = &buffer.len();
    Woz::push_32(&mut buffer, 0);   // chunk size, we'll only know wiat it is later
    let mut block: u16 = 3;
    let mut encoded_track_in_bits: Vec<Vec<u8>> = Vec::new();
    for track in 0..35 {
        Woz::push_16(&mut buffer, block);
        Woz::push_16(&mut buffer, 13);
        block += 13;
        let this_track = &buffer[track * 256..(track + 1) * 256 * 16];
        let encoded = Dsk::encode_track(this_track, track as u8);
        Woz::push_32(&mut buffer, encoded.len() as u32);  // bit count
        encoded_track_in_bits.push(encoded);
    }
    for _ in 35..160 {
        Woz::push_16(&mut buffer, 0);
        Woz::push_16(&mut buffer, 0);
        Woz::push_32(&mut buffer, 0);  // bit count
    }

    for track in encoded_track_in_bits {
        let mut i = 0;
        // Need to fill exactly 13 blocks
        let mut total = 13 * 512;
        // println!("track len: {}", track.len());
        while i < track.len() {
            let mut n = 0;
            let mut a = 0;
            while i < track.len() && n < 8 {
                let bit = track[i];
                a = (a << 1) | bit;
                i += 1;
                n += 1;
            }
            if n != 8 {
                panic!("Ending on a non byte boundary");
            }
            buffer.push(a);
            total -= 1;
        }
        // Pad the rest of the content to the block boundary
        while total > 0 {
            buffer.push(0);
            total -= 1;
        }

    }
    // Write the size now that we know it
    let chunk_size = buffer.len() - *chunk_size_index - 4;
    set_32(&mut buffer, *chunk_size_index, chunk_size as u32);

    // Done with the file, calculate the checksum and save it
    let checksum = crc32(0, &buffer[12..]);
    set_32(&mut buffer, 8, checksum);

    // Save the file
    let stem = Path::new(path).file_stem().unwrap().to_str().unwrap();
    let file_out = "bad.woz".to_string(); // format!("{}.woz", stem);
    let mut file = File::create(&file_out).map_err(|e| e.to_string())?;
    file.write(&buffer).map(|_| file_out).map_err(|e| e.to_string())
}

fn set_32(buffer: &mut [u8], index: usize, value: u32) {
    buffer[index] = (value & 0xff) as u8;
    buffer[index + 1] = ((value & 0xff00) >> 8) as u8;
    buffer[index + 2] = ((value & 0xffff00) >> 16) as u8;
    buffer[index + 3] = ((value & 0xff000000) >>24) as u8;
}

fn push_string(buffer: &mut Vec<u8>, s: &str) {
    s.chars().for_each(|c| buffer.push(c as u8));
}

fn push_bytes(buffer: &mut Vec<u8>, a: &[u8]) {
    for b in a.iter() {
        buffer.push(*b);
    }
}

const CRC32_TAB: [u32;256] = [
    0x00000000, 0x77073096, 0xee0e612c, 0x990951ba, 0x076dc419, 0x706af48f,
    0xe963a535, 0x9e6495a3, 0x0edb8832, 0x79dcb8a4, 0xe0d5e91e, 0x97d2d988,
    0x09b64c2b, 0x7eb17cbd, 0xe7b82d07, 0x90bf1d91, 0x1db71064, 0x6ab020f2,
    0xf3b97148, 0x84be41de, 0x1adad47d, 0x6ddde4eb, 0xf4d4b551, 0x83d385c7,
    0x136c9856, 0x646ba8c0, 0xfd62f97a, 0x8a65c9ec, 0x14015c4f, 0x63066cd9,
    0xfa0f3d63, 0x8d080df5, 0x3b6e20c8, 0x4c69105e, 0xd56041e4, 0xa2677172,
    0x3c03e4d1, 0x4b04d447, 0xd20d85fd, 0xa50ab56b, 0x35b5a8fa, 0x42b2986c,
    0xdbbbc9d6, 0xacbcf940, 0x32d86ce3, 0x45df5c75, 0xdcd60dcf, 0xabd13d59,
    0x26d930ac, 0x51de003a, 0xc8d75180, 0xbfd06116, 0x21b4f4b5, 0x56b3c423,
    0xcfba9599, 0xb8bda50f, 0x2802b89e, 0x5f058808, 0xc60cd9b2, 0xb10be924,
    0x2f6f7c87, 0x58684c11, 0xc1611dab, 0xb6662d3d, 0x76dc4190, 0x01db7106,
    0x98d220bc, 0xefd5102a, 0x71b18589, 0x06b6b51f, 0x9fbfe4a5, 0xe8b8d433,
    0x7807c9a2, 0x0f00f934, 0x9609a88e, 0xe10e9818, 0x7f6a0dbb, 0x086d3d2d,
    0x91646c97, 0xe6635c01, 0x6b6b51f4, 0x1c6c6162, 0x856530d8, 0xf262004e,
    0x6c0695ed, 0x1b01a57b, 0x8208f4c1, 0xf50fc457, 0x65b0d9c6, 0x12b7e950,
    0x8bbeb8ea, 0xfcb9887c, 0x62dd1ddf, 0x15da2d49, 0x8cd37cf3, 0xfbd44c65,
    0x4db26158, 0x3ab551ce, 0xa3bc0074, 0xd4bb30e2, 0x4adfa541, 0x3dd895d7,
    0xa4d1c46d, 0xd3d6f4fb, 0x4369e96a, 0x346ed9fc, 0xad678846, 0xda60b8d0,
    0x44042d73, 0x33031de5, 0xaa0a4c5f, 0xdd0d7cc9, 0x5005713c, 0x270241aa,
    0xbe0b1010, 0xc90c2086, 0x5768b525, 0x206f85b3, 0xb966d409, 0xce61e49f,
    0x5edef90e, 0x29d9c998, 0xb0d09822, 0xc7d7a8b4, 0x59b33d17, 0x2eb40d81,
    0xb7bd5c3b, 0xc0ba6cad, 0xedb88320, 0x9abfb3b6, 0x03b6e20c, 0x74b1d29a,
    0xead54739, 0x9dd277af, 0x04db2615, 0x73dc1683, 0xe3630b12, 0x94643b84,
    0x0d6d6a3e, 0x7a6a5aa8, 0xe40ecf0b, 0x9309ff9d, 0x0a00ae27, 0x7d079eb1,
    0xf00f9344, 0x8708a3d2, 0x1e01f268, 0x6906c2fe, 0xf762575d, 0x806567cb,
    0x196c3671, 0x6e6b06e7, 0xfed41b76, 0x89d32be0, 0x10da7a5a, 0x67dd4acc,
    0xf9b9df6f, 0x8ebeeff9, 0x17b7be43, 0x60b08ed5, 0xd6d6a3e8, 0xa1d1937e,
    0x38d8c2c4, 0x4fdff252, 0xd1bb67f1, 0xa6bc5767, 0x3fb506dd, 0x48b2364b,
    0xd80d2bda, 0xaf0a1b4c, 0x36034af6, 0x41047a60, 0xdf60efc3, 0xa867df55,
    0x316e8eef, 0x4669be79, 0xcb61b38c, 0xbc66831a, 0x256fd2a0, 0x5268e236,
    0xcc0c7795, 0xbb0b4703, 0x220216b9, 0x5505262f, 0xc5ba3bbe, 0xb2bd0b28,
    0x2bb45a92, 0x5cb36a04, 0xc2d7ffa7, 0xb5d0cf31, 0x2cd99e8b, 0x5bdeae1d,
    0x9b64c2b0, 0xec63f226, 0x756aa39c, 0x026d930a, 0x9c0906a9, 0xeb0e363f,
    0x72076785, 0x05005713, 0x95bf4a82, 0xe2b87a14, 0x7bb12bae, 0x0cb61b38,
    0x92d28e9b, 0xe5d5be0d, 0x7cdcefb7, 0x0bdbdf21, 0x86d3d2d4, 0xf1d4e242,
    0x68ddb3f8, 0x1fda836e, 0x81be16cd, 0xf6b9265b, 0x6fb077e1, 0x18b74777,
    0x88085ae6, 0xff0f6a70, 0x66063bca, 0x11010b5c, 0x8f659eff, 0xf862ae69,
    0x616bffd3, 0x166ccf45, 0xa00ae278, 0xd70dd2ee, 0x4e048354, 0x3903b3c2,
    0xa7672661, 0xd06016f7, 0x4969474d, 0x3e6e77db, 0xaed16a4a, 0xd9d65adc,
    0x40df0b66, 0x37d83bf0, 0xa9bcae53, 0xdebb9ec5, 0x47b2cf7f, 0x30b5ffe9,
    0xbdbdf21c, 0xcabac28a, 0x53b39330, 0x24b4a3a6, 0xbad03605, 0xcdd70693,
    0x54de5729, 0x23d967bf, 0xb3667a2e, 0xc4614ab8, 0x5d681b02, 0x2a6f2b94,
    0xb40bbe37, 0xc30c8ea1, 0x5a05df1b, 0x2d02ef8d
];

pub fn crc32(mut crc: u32, bytes: &[u8]) -> u32 {
    crc ^= 0xffffffff;
    for byte in bytes {
        crc = CRC32_TAB[((crc ^ *byte as u32) & 0xff) as usize] ^ (crc >> 8);
    }
    crc ^ 0xffffffff
}



