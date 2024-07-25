use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use lazy_static::lazy_static;
use crate::disk::disk::PDisk;
use crate::disk::disk_controller::MAX_PHASE;
use crate::disk::disk_info::DiskInfo;
use crate::disk::woz::Woz;

lazy_static! {
    pub static ref ALL_DISKS: Vec<DiskInfo> = vec![
        DiskInfo::new("DOS 3.3", "d:\\Apple disks\\Apple DOS 3.3.dsk"), // 0
        DiskInfo::new("Dos 3.3 August 1980", "d:\\Apple disks\\Apple DOS 3.3 August 1980.dsk"), // 1
        DiskInfo::new("NTSC", "d:\\Apple disks\\ntsc.dsk"), // 2
        DiskInfo::new("master", "d:\\Apple disks\\master.dsk"), // 3
        DiskInfo::new("Sherwood Forest", "d:\\Apple disks\\Sherwood_Forest.dsk"),  // 4
        DiskInfo::new("A2Audit", "C:\\Users\\Ced\\kotlin\\sixty\\src\\test\\resources\\audit.dsk"), // 5
        DiskInfo::new("ProDOS 2.4.1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\ProDOS_2_4_1.dsk"), // 6
        DiskInfo::new("Cedric", "d:\\Apple disks\\cedric.dsk"), // 7
        DiskInfo::new("Transylvania *", "d:\\Apple disks\\TRANS1.DSK"), // 8
        DiskInfo::new("Masquerade - 1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Masquerade-1.dsk"), // 9
        DiskInfo::new("Masquerade - 2", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Masquerade-2.dsk"), // 10
        DiskInfo::new("Ultima 4 - 1", "C:\\Users\\Ced\\kotlin\\sixty\\disks\\Ultima4.dsk"), // 11
        DiskInfo::new("Blade of Blackpoole - 1", "d:\\Apple disks\\The Blade of Blackpoole side A.woz"), // 12
        DiskInfo::new("Blade of Blackpoole - 2", "d:\\Apple disks\\The Blade of Blackpoole side B.woz"), // 13
        DiskInfo::new("The Coveted Mirror - 1", "d:\\Apple disks\\COVETED1.DSK"), // 14
        DiskInfo::new("The Coveted Mirror - 2", "d:\\Apple disks\\COVETED2.DSK"), // 15
        DiskInfo::new("Sherwood Forest", "d:\\Apple disks\\Sherwood Forest.woz"), // 16
        DiskInfo::new("Apple DOS 3.3", "d:\\Apple disks\\DOS 3.3.woz"), // 17
        DiskInfo::new("Blazing Paddles", "d:\\Apple disks\\Blazing Paddles (Baudville).woz"), // 18
        DiskInfo::new("Bouncing Kamungas", "d:\\Apple disks\\Bouncing Kamungas.woz"), // 19
        DiskInfo::new("Commando - 1", "d:\\Apple disks\\Commando - DIsk 1, Side A.woz"), // 20
        DiskInfo::new("Night Mission Pinball", "d:\\Apple disks\\Night Mission Pinball.woz"), // 21
        DiskInfo::new("Rescue Raiders", "d:\\Apple disks\\Rescue Raiders - Disk 1, Side B.woz"), // 22
        DiskInfo::new("Karateka", "d:\\Apple disks\\Karateka.dsk"), // 23
        DiskInfo::new("Dark Lord - 1", "d:\\Apple disks\\Dark Lord side A.woz"), // 24
        DiskInfo::new("Dark Lord - 2", "d:\\Apple disks\\Dark Lord side B.woz"), // 25
        DiskInfo::new("Sammy Lightfoot", "d:\\Apple disks\\Sammy Lightfoot - Disk 1, Side A.woz"), // 26
        DiskInfo::new("Stargate - 1 *", "d:\\Apple disks\\Stargate - Disk 1, Side A.woz"), // 27
        DiskInfo::new("Stellar 7", "d:\\Apple disks\\Stellar 7.woz"), // 28
        DiskInfo::new("Aztec", "d:\\Apple disks\\Aztec (4am crack).dsk"), // 29
        DiskInfo::new("Aztec", "d:\\Apple disks\\Aztec.woz"), // 30
        DiskInfo::new("Conan - 1", "d:\\Apple disks\\Conan side A.woz"), // 31
        DiskInfo::new("Conan - 2", "d:\\Apple disks\\Conan side B.woz"), // 32
        DiskInfo::new("Adventureland - 1", "d:\\Apple disks\\Adventureland - 1.woz"), // 33
        DiskInfo::new("Adventureland - 2", "d:\\Apple disks\\Adventureland - 2.woz"), // 34
        DiskInfo::new("Arctic Fox", "d:\\Apple disks\\Arcticfox.woz"), // 35
        DiskInfo::new("Frogger", "d:\\Apple disks\\Frogger.woz"), // 36
        DiskInfo::new("Demo by Five Touch", "d:\\Apple disks\\patched.woz"), // 37
        DiskInfo::new("Wizardry 1", "d:\\Apple disks\\Wizardry 1 - 1.woz"), // 38
        DiskInfo::new("a2fc", "d:\\Apple disks\\A2BestPix_Top1.DSK"), // 39
        DiskInfo::new("King's Quest 1 - 1", "d:\\Apple disks\\King's Quest - A.woz"), // 40
        DiskInfo::new("test", "c:\\Users\\Ced\\rust\\sixty.rs\\bad.woz"), // 41
        DiskInfo::new("Star Trek - 1", "d:\\Apple disks\\Star Trek - First Contact - Disk 1, Side 1.woz"), // 42
        DiskInfo::n("d:\\Apple disks\\Airheart.woz"), // 43
        DiskInfo::n("d:\\Apple disks\\Apple Galaxian.woz"), // 44
    ];
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
