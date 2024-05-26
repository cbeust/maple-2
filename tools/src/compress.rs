use std::fs::File;
use std::io::Read;

pub fn compress() {
    let mut file = File::open("d:\\t\\pic.hgr").unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let mut compressed: Vec<u8> = Vec::new();
    let mut current = buffer[0];
    let mut count = 1;
    for i in 1..0x2000 {
        if buffer[i] != current {
            // println!("Next {:02X} != current {:02X}", buffer[i], current);
            compressed.push(count);
            compressed.push(current);
            if count != 1 {
                println!("Encoding [{:04X}] {:02X} {:02X}", (i - count as usize), count, current);
            }
            count = 1;
            current = buffer[i];
        } else {
            count += 1;
        }
    }
    println!("End compression, size: original: {} {}", buffer.len(), compressed.len());
}
