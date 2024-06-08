use cpu::memory::Memory;
use crate::apple2_cpu::EmulatorConfigMsg;
use crate::create_apple2;
use crate::disk::disk_controller::{DiskController};
use crate::disk::disk_info::DiskInfo;
use crate::disk::dsk::Dsk;
use crate::memory::Apple2Memory;

#[test]
fn test_write8() {
    let mut buffer: Vec<u8> = Vec::new();
    Dsk::write8(&mut buffer, vec![0b1101_1011]);
    let expected: Vec<u8> = vec![1, 1, 0, 1, 1, 0, 1, 1];
    assert_eq!(buffer.len(), expected.len());
    for i in 0..buffer.len() {
        assert_eq!(buffer[i], expected[i]);
    }
}

#[test]
fn test_write_sync() {
    let mut buffer: Vec<u8> = Vec::new();
    Dsk::write_sync(&mut buffer, 2);
    let expected: Vec<u8> = vec![1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0];
    assert_eq!(buffer.len(), expected.len());
    for i in 0..buffer.len() {
        assert_eq!(buffer[i], expected[i]);
    }
}

// #[cfg(test)]
const SECTOR: [u8; 256] = [
    0x01, 0xA5, 0x27, 0xC9, 0x09, 0xD0, 0x18, 0xA5, 0x2B, 0x4A, 0x4A, 0x4A, 0x4A, 0x09, 0xC0, 0x85,
    0x3F, 0xA9, 0x5C, 0x85, 0x3E, 0x18, 0xAD, 0xFE, 0x08, 0x6D, 0xFF, 0x08, 0x8D, 0xFE, 0x08, 0xAE,
    0xFF, 0x08, 0x30, 0x15, 0xBD, 0x4D, 0x08, 0x85, 0x3D, 0xCE, 0xFF, 0x08, 0xAD, 0xFE, 0x08, 0x85,
    0x27, 0xCE, 0xFE, 0x08, 0xA6, 0x2B, 0x6C, 0x3E, 0x00, 0xEE, 0xFE, 0x08, 0xEE, 0xFE, 0x08, 0x20,
    0x89, 0xFE, 0x20, 0x93, 0xFE, 0x20, 0x2F, 0xFB, 0xA6, 0x2B, 0x6C, 0xFD, 0x08, 0x00, 0x0D, 0x0B,
    0x09, 0x07, 0x05, 0x03, 0x01, 0x0E, 0x0C, 0x0A, 0x08, 0x06, 0x04, 0x02, 0x0F, 0x48, 0x20, 0x64,
    0xA7, 0xB0, 0x05, 0xA9, 0x00, 0xA8, 0x91, 0x40, 0x68, 0x4C, 0xD2, 0xA6, 0xAD, 0xE6, 0xB5, 0xD0,
    0x0B, 0xAD, 0xE4, 0xB5, 0xD0, 0x03, 0xCE, 0xE5, 0xB5, 0xCE, 0xE4, 0xB5, 0xCE, 0xE6, 0xB5, 0x4C,
    0x7E, 0xAE, 0x20, 0x94, 0xB1, 0x4C, 0xBE, 0xA6, 0x20, 0xA3, 0xA2, 0xAD, 0xEE, 0xB5, 0xA8, 0x0A,
    0xAD, 0xEF, 0xB5, 0xAA, 0x2A, 0x69, 0x01, 0x0A, 0x85, 0x42, 0x98, 0xE5, 0x42, 0x8D, 0xE4, 0xB5,
    0x8A, 0xE9, 0x00, 0x8D, 0xE5, 0xB5, 0xB0, 0xD7, 0x60, 0xC0, 0x01, 0xAD, 0xE6, 0xB5, 0xD0, 0x37,
    0xAD, 0xF6, 0xB5, 0xF0, 0x32, 0xAD, 0xC2, 0xB5, 0xF0, 0x2D, 0x90, 0x1D, 0xA9, 0xAD, 0x20, 0xF1,
    0x9D, 0x69, 0x80, 0x08, 0x20, 0xC6, 0xB0, 0xB0, 0x26, 0x28, 0x90, 0x05, 0x20, 0x52, 0xB1, 0x70,
    0x05, 0x20, 0x73, 0xB1, 0xF0, 0x09, 0x20, 0x99, 0xB1, 0x20, 0xFB, 0x9D, 0xB8, 0x50, 0xE4, 0x20,
    0xF7, 0x9D, 0xB0, 0x08, 0x20, 0x99, 0xB1, 0xB0, 0x03, 0x4C, 0x96, 0xAC, 0x4C, 0xCA, 0xAC, 0x4C,
    0x6F, 0xB3, 0xA3, 0xA0, 0xD2, 0xCF, 0xD2, 0xD2, 0xC5, 0x8D, 0x87, 0x8D, 0x00, 0x00, 0xB6, 0x09,
];

// #[test]
// fn test_c600() {
//     let buffer = {
//         let mut dc = DiskController::new_with_filename(6, &[Some(DiskInfo::n("files/master.dsk")), None], None);
//         let mut buffer: Vec<u8> = Vec::new();
//         for i in 0..TRACK_SIZE_BYTES * 2 {
//             buffer.push(dc.next_byte());
//         }
//         buffer
//     };
//
//     // crate::debug::hex_dump(&buffer);
//
//     use std::ops::Shl;
//
//     fn w(a:u8, b:u8) -> u8 {
//         let bit = (a & 1 << 7) >> 7;
//         let result = ((a as u16).shl(1) | bit as u16) & b as u16;
//         // println!("Decoding {:02X} {:02X}: {:02X}", a, b, result);
//         result as u8
//     }
//
//     let mut i = 0;
//     while i < buffer.len() {
//         let b = buffer[i];
//         if b == 0xd5 && buffer[i + 1] == 0xaa && buffer[i + 2] == 0x96 {
//             let volume = w(buffer[i + 3], buffer[i + 4]);
//             let track = w(buffer[i + 5], buffer[i + 6]);
//             let sector = w(buffer[i + 7], buffer[i + 8]);
//             let checksum = w(buffer[i + 9], buffer[i + 10]);
//             println!("{} V:{:02X} T:{:02X} S:{:02X} C:{:02X}", i, volume, track, sector, checksum);
//             if sector == 0 {
//                 println!("Found sector 0 at {}", i);
//             }
//             // println!("  fields: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
//             //          buffer[i+5], buffer[i+6],
//             //          buffer[i+7], buffer[i+8],
//             //          buffer[i+9], buffer[i+10]);
//             i += 10;
//         } else {
//             // println!("Skipping {:02X}",b);
//             i += 1;
//         }
//     }
// }

///
/// Test just the encoded data
///
#[test]
fn test_encoded_data() {
    let expected_encoded = vec![
        0xb6, 0xf3, 0xdc, 0xf4, 0xb9, 0xf5, 0xf7, 0xeb, 0xb5, 0xef, 0xfc, 0xfc, 0xdf, 0xda, 0xe6,
        0xd9, 0xab, 0xab, 0xd9
    ];
    let encoded = Dsk::encode_6and2(&SECTOR);
    for i in 0..expected_encoded.len() {
        assert_eq!(encoded[i], expected_encoded[i]);
    }
}


///
/// Test the encoding of the track, up to the beginning of the data section
///
// #[test]
// fn test_write_track() {
//     let expected_encoded = vec![
//         0xb6, 0xf3, 0xdc, 0xf4, 0xb9, 0xf5, 0xf7, 0xeb, 0xb5, 0xef, 0xfc, 0xfc, 0xdf, 0xda, 0xe6,
//         0xd9, 0xab, 0xab, 0xd9
//     ];
//     // Create a disk made of 40 * 16 sectors all identical
//     let mut disk: Vec<u8> = Vec::new();
//     for _ in 0..PHASES {
//         for _ in 0..16 {
//             for i in 0..SECTOR.len() {
//                 disk.push(SECTOR[i]);
//             }
//         }
//     }
//
//     let mut dc = Woz::new(&disk, ""); //DiskController::new_with_bytes(6, &[None, None], [Some(disk), None], None);
//     let expected_track = vec![
//         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD5, 0xAA, 0x96, 0xFF,
//         0xFE, 0xAA, 0xAA, 0xAA, 0xAA, 0xFF, 0xFE, 0xDE, 0xAA, 0xEB,
//         0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xD5, 0xAA, 0xAD,
//         expected_encoded[0], expected_encoded[1], expected_encoded[2]
//     ];
//
//     for i in 0..expected_track.len() { // expected_track.len() {
//         let b = dc.unwrap().bit_streams().get_stream(i * 4).next_byte();
//         assert_eq!(b, expected_track[i], "Index {}, expected {:02X}, got {:02X}",
//                    i, expected_track[i], b);
//     }
// }

// #[test]
fn test_boot_sequence() {
    let mut computer = create_apple2::<Apple2Memory>(None, None, None, [None, None],
        EmulatorConfigMsg::default(), None);
    println!("Created computer");
    computer.cpu.cpu.pc = 0xc65c;
    computer.cpu.cpu.x = 0x60;
    let motor_on = computer.cpu.cpu.memory.get(0xc0e9);
    let mut stop = false;
    while ! stop {
        computer.cpu.step();
        // if computer.cpu.cpu.a == 0xd5 {
        //     println!("FOUND d5");
        // }
        if computer.cpu.cpu.pc == 0xc6f8 {
            println!("Done booting");
            computer.cpu.cpu.memory.dump_at(0x800, 0x100);
            stop = true;
        }
    }
}

pub(crate) fn test_bit_buffer() {
    let disk_info = DiskInfo::n("D:\\PD\\Apple disks\\Apple DOS 3.3.dsk");
    let mut dc = DiskController::new_with_filename(6, &[Some(disk_info), None], None);
    // dc.set_track(4);
    // dc.set_bit_position(22759);
    println!("Next byte: {:02X}", dc.next_byte());
    // dc.set_bit_position(22760);
    println!("Next byte: {:02X}", dc.next_byte());
    // dc.set_bit_position(22761);
    println!("Next byte: {:02X}", dc.next_byte());
}