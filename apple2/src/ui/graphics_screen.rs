use eframe::egui::{Color32, Pos2, Rect, Ui};
use crate::constants::MAGNIFICATION;
use crate::misc::bit;
use crate::ui::hires_screen::{AColor::*, *};
use crate::ui::ui::{DrawCommand, MyEguiApp};
use crate::ui::ui::DrawCommand::Rectangle;

pub fn test_hires_colors_sliding_window() {
    let data = [
        ([
            0, 0, 0, 0, 0, 0, 1,
            1, 1, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ], [0, 0, 0, 2, 3, 11, 11, 9, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    ];

    for (bits, expected) in data {
        let colors = MyEguiApp::calculate_double_hires_colors_from_bits(bits);
        let expected_colors = expected.iter().map(|c| AColor::to_double_hires_color(*c as u8)).collect::<Vec<AColor>>();
        assert_eq!(colors, expected_colors);
    }
}

impl MyEguiApp {
    /// The 32 bits correspond to:
    /// 2 bits on the left
    /// 7 bits * 4 extracted from the memory locations (e.g. AUX $2000, MAIN $2000, AUX $2001, MAIN $2001),
    /// excluding bit 7 of each of the 4 bytes (that palette bit is ignored in double hires)
    /// 2 bits on the right
    /// Then apply the sliding window algorithm described in:
    /// https://docs.google.com/presentation/d/1kiHLrvObm2IyCYLCPEaAqdPEAAeScZcsEBP6_r5BVow/edit#slide=id.ge2ed09e2a4_0_162
    ///
    /// Return a vector of 28 colors.
    pub fn calculate_double_hires_colors_from_bits(bits: [u8; 32]) -> Vec<AColor> {
        let mut result: Vec<AColor> = Vec::new();
        let mut index = 0;
        while index < bits.len() - 4 {
            let md = index % 4;
            let color =
                // if true {
                    if md == 0 {
                        // println!("Index mod = 0, picking indices {} {} {} {}", index, index+1, index+2, index+3);
                        bits[index + 0] << 3 | bits[index + 1] << 2 | bits[index + 2] << 1 | bits[index + 3]
                    } else if md == 1 {
                        // println!("Index mod = 1, picking indices {} {} {} {}",  index+3, index, index+1, index+2);
                        bits[index + 3] << 3 | bits[index + 0] << 2 | bits[index + 1] << 1 | bits[index + 2]
                    } else if md == 2 {
                        // println!("Index mod = 2, picking indices {} {} {} {}", index+2, index+3, index, index+1);
                        bits[index + 2] << 3 | bits[index + 3] << 2 | bits[index + 0] << 1 | bits[index + 1]
                    } else {
                        // println!("Index mod = 3, picking indices {} {} {} {}", index+1, index+2, index+3, index);
                        bits[index + 1] << 3 | bits[index + 2] << 2 | bits[index + 3] << 1 | bits[index + 0]
                    };
                // } else {
                //     bits[index] << 3 | bits[index + 1] << 2 | bits[index + 2] << 1 | bits[index + 3];
                // }

            // println!("Index {index} {color} color:{:#?}", AColor::to_double_hires_color(color));
            result.push(AColor::to_double_hires_color(color));
            index += 1;
        }
        result
    }

    pub fn bits(byte_left: u8, byte0: u8, byte1: u8, byte2: u8, byte3: u8, byte_right: u8) -> [u8; 32] {
        [
            bit(byte_left, 5), bit(byte_left, 6),
            bit(byte0, 0), bit(byte0, 1), bit(byte0, 2), bit(byte0, 3), bit(byte0, 4), bit(byte0, 5), bit(byte0, 6),
            bit(byte1, 0), bit(byte1, 1), bit(byte1, 2), bit(byte1, 3), bit(byte1, 4), bit(byte1, 5), bit(byte1, 6),
            bit(byte2, 0), bit(byte2, 1), bit(byte2, 2), bit(byte2, 3), bit(byte2, 4), bit(byte2, 5), bit(byte2, 6),
            bit(byte3, 0), bit(byte3, 1), bit(byte3, 2), bit(byte3, 3), bit(byte3, 4), bit(byte3, 5), bit(byte3, 6),
            bit(byte_right, 0), bit(byte_right, 1),
        ]
    }

    pub fn calculate_double_hires2(&mut self, page2: bool) -> Vec<DrawCommand> {
        let mut result: Vec<DrawCommand> = Vec::new();
        let hgr = Hgr::default();
        let start = if page2 { 0x4000 } else { 0x2000 };

        for (y, a) in hgr.enumerate() {
            let mut x = 0;
            let mut address = a;
            while x < 560 {
                let commands = self.get_double_hires_draw_commands(x as u16, y as u16, start + address);
                x += commands.len();
                result.extend(commands);
                address += 1;
            }
        }

        result
    }

    /// Shamelessly copied from AppleWin's implementation
    fn get_double_hires_draw_commands(&mut self, x: u16, y: u16, mut address: usize)
            -> Vec<DrawCommand> {
        /// "Mixed mode" is really black and white mode, only happens for a combination
        /// of the F1 and F2 switches "10" == 2. See [crate::memory::update_f1_f2()] for details.
        let is_mixed_mode = self.dhg_rgb_mode == 2;
        let mut result: Vec<DrawCommand> = Vec::new();
        let x_offset = address & 1;
        address -= x_offset;

        let byteval1 = self.cpu.aux_memory[address] as u32;
        let byteval2 = self.cpu.memory[address] as u32;
        let byteval3 = self.cpu.aux_memory[address + 1] as u32;
        let byteval4 = self.cpu.memory[address + 1] as u32;
        let mut dwordval = (byteval1 & 0x7f)
            | (byteval2 & 0x7f) << 7
            | (byteval3 & 0x7f) << 14
            | (byteval4 & 0x7f) << 21;
        let mut colors: [u8; 7] = [0; 7];
        let mut bits: [u8; 7] = [0; 7];
        let mut dwordval_tmp = dwordval;
        for i in 0..7 {
            bits[i] = (dwordval_tmp & 0xf) as u8;
            colors[i] = ((bits[i] & 7) << 1) | ((bits[i] & 8) >> 3);
            dwordval_tmp >>= 4;
        }
        let bw: [u8; 2] = [
            0, 15,
        ];

        let mut x_index: usize = 0;

        let mut push = |c: u8| {
            let color = AColor::to_double_hires_color(c);
            let mag = MAGNIFICATION / 2;
            let x0 = (x_index + x as usize) * mag as usize;
            x_index += 1;
            let x1 = x0 + mag as usize;
            let y0 = y as usize * MAGNIFICATION as usize;
            let y1 = y0 + MAGNIFICATION as usize;

            // println!("Drawing {},{} {},{} color:{:#?}", x0, y0, x1, y1, color);

            result.push(Rectangle(x0 as f32, y0 as f32, x1 as f32, y1 as f32, color));
        };

        if x_offset == 0 {
            // First cell (address is even)
            if (byteval1 & 0x80) > 0 || !is_mixed_mode {
                // Cell 0
                push(colors[0]);
                push(colors[0]);
                push(colors[0]);
                push(colors[0]);

                // Cell 1
                push(colors[1]);
                push(colors[1]);
                push(colors[1]);
                dwordval >>= 7;
                self.last_cell_is_color = true;
            } else {
                for _ in 0..7 {
                    self.last_bit = (dwordval & 1) as usize;
                    push(bw[self.last_bit]);
                    dwordval >>= 1;
                }
                self.last_cell_is_color = false;
            }
            // Cell 1, 2, and 3
            if (byteval2 & 0x80) > 0 || !is_mixed_mode {
                // Remaining of cell 1
                if self.last_cell_is_color {
                    push(colors[1]);
                } else {
                    push(bw[self.last_bit]);
                }

                // Cell 2
                push(colors[2]);
                push(colors[2]);
                push(colors[2]);
                push(colors[2]);

                // Cell 3
                push(colors[3]);
                push(colors[3]);
                self.last_cell_is_color = true;

            } else {
                for _ in 0..7 {
                    self.last_bit = (dwordval & 1) as usize;
                    push(bw[self.last_bit]);
                    dwordval >>= 1;
                }
                self.last_cell_is_color = false;
            }
        } else {
            // Second cell (x is odd)
            dwordval >>= 14;
            if (byteval3 & 0x80) > 0 || !is_mixed_mode {
                if self.last_cell_is_color {
                    // Finish cell 3
                    push(colors[3]);
                    push(colors[3]);
                } else {
                    push(bw[self.last_bit]);
                    push(bw[self.last_bit]);
                }

                // Cell 4
                push(colors[4]);
                push(colors[4]);
                push(colors[4]);
                push(colors[4]);

                // Cell 5
                push(colors[5]);

                dwordval >>= 7;
                self.last_cell_is_color = true;
            } else {
                for _ in 0..7 {
                    self.last_bit = (dwordval & 1) as usize;
                    push(bw[self.last_bit]);
                    dwordval >>= 1;
                }
                self.last_cell_is_color = false;
            }

            if (byteval4 & 0x80) > 0 || !is_mixed_mode {
                if self.last_cell_is_color {
                    // Cell 5
                    push(colors[5]);
                    push(colors[5]);
                    push(colors[5]);
                } else {
                    push(bw[self.last_bit]);
                    push(bw[self.last_bit]);
                    push(bw[self.last_bit]);
                }

                // Cell 6
                push(colors[6]);
                push(colors[6]);
                push(colors[6]);
                push(colors[6]);
                self.last_cell_is_color = true;
            } else {
                for _ in 0..7 {
                    self.last_bit = (dwordval & 1) as usize;
                    push(bw[self.last_bit]);
                    dwordval >>= 1;
                }
                self.last_cell_is_color = false;
            }
        }

        result
    }

    pub fn calculate_double_hires(&self, page2: bool) -> Vec<DrawCommand> {
        let mut result: Vec<DrawCommand> = Vec::new();
        let start = if page2 { 0x4000 } else { 0x2000 };

        // Calculate pixel colors by mixing aux and main memory bytes
        let mut x = 0;
        let hgr = Hgr::default();
        for (y, a) in hgr.enumerate() {
            for byte_index in (0..0x28).step_by(2) {
                let a2 = start + a + byte_index;
                let mask = a2 & 0xff;
                let byte0 = self.cpu.aux_memory[a2];
                let byte1 = self.cpu.memory[a2];
                let byte2 = self.cpu.aux_memory[a2 + 1];
                let byte3 = self.cpu.memory[a2 + 1];

                // Beginning of line?
                let beginning = [0, 0x80, 0x28, 0xa8, 0x50, 0x50].contains(&mask);
                // End of line? (-1 the actual end of line since we look at bytes as pairs)
                let end = [0x26, 0xa6, 0x4e, 0xce, 0x76, 0xf6].contains(&mask);

                let left_byte = if beginning { 0 } else { self.cpu.memory[a2 - 1] };
                let right_byte = if end { 0 } else { self.cpu.aux_memory[a2 + 2] };

                let bits = MyEguiApp::bits(left_byte, byte0, byte1, byte2, byte3, right_byte);
                let colors = MyEguiApp::calculate_double_hires_colors_from_bits(bits);

                let mag = 1.0; // MAGNIFICATION as f32;
                let w = 1.0;
                let h = 3.0;
                let y0 = (y as f32 * (h + 1.0)) * mag;
                for color in colors {
                    let x0 = (x as f32 * (w + 1.0)) * mag;
                    let x1 = x0 + w;
                    let y1 = (y0 + h) * mag;
                   // println!("A: {a2:04X} {x},{y} Rect:{x0},{y0} - {x1},{y1} {color:#?}");
                    result.push(Rectangle(x0, y0, x1, y1, color));
                    // println!("Rectangle: a:{a:04X} {x0},{y0} - {x1},{y1}  {color:#?}");
                    x += 1;
                }
            }
            x = 0;
        }
        result
    }

    pub fn calculate_hires(&mut self, height: u16, page2: bool) -> Vec<DrawCommand> {
        let memory = if page2 { &self.cpu.memory[0x4000..0x6000] }
        else { &self.cpu.memory[0x2000..0x4000] };
        let pixels = calculate_pixels(memory);
        let mut result: Vec<DrawCommand> = Vec::new();
        let mag = MAGNIFICATION as f32;
        for pixel in pixels {
            if pixel.y < height {
                let x = pixel.x as f32 * mag;
                let y = pixel.y as f32 * mag;
                result.push(Rectangle(x, y, x + mag, y+ 2.0, pixel.color));
                // Use y + 2.0 here to create a banding effect, use y + MAGNIFICATION for brighter
                // rect_filled(ui, x, y, x + mag, y + 2.0, pixel.color.into());
            }
        }

        result
    }
}

/// Move a sliding window of three bits (previous, current, next) and calculate
/// the colors from that triplet.
pub fn calculate_correct_colors_from_bytes(left_bit: u8, b0: u8, b1: u8, right_bit: u8)
    -> Vec<AColor> {
    let mut result: Vec<AColor> = Vec::new();
    let high_bit0 = bit(b0, 7);
    let high_bit1 = bit(b1, 7);
    let bits = [
        left_bit,
        bit(b0, 0), bit(b0, 1), bit(b0, 2), bit(b0, 3), bit(b0, 4), bit(b0, 5), bit(b0, 6),
        bit(b1, 0), bit(b1, 1), bit(b1, 2), bit(b1, 3), bit(b1, 4), bit(b1, 5), bit(b1, 6),
        right_bit
    ];
    let mut i = 0;
    let mut even = true;
    let mut high_bit = high_bit0;
    while i < bits.len() - 2 {
        result.push(correct_color(high_bit, even, bits[i], bits[i + 1], bits[i + 2]));
        if i >= 7 { high_bit = high_bit1; }
        even = ! even;
        i += 1;
    }
    result
}

/// Return all the pixels for this area of memory (0..0x2000)
pub fn calculate_pixels(memory: &[u8]) -> Vec<Pixel> {
    let mut result: Vec<Pixel> = Vec::new();
    let screen = HiresScreen::new();
    for i in (0..memory.len()).step_by(2) {
        let mask = i & 0xff;
        if (mask >= 0x78 && mask <= 0x7f) || (mask >=0xf8 && mask <= 0xff) {
            // Skip unused parts of the graphic memory
            continue;
        }
        // Beginning of line?
        let beginning = mask == 0 || mask == 0x80 || mask == 0x28 || mask == 0xa8 || mask == 0x50 || mask == 0xd0;
        // End of line? (-1 the actual end of line since we look at bytes as pairs)
        let end = mask == 0x26 || mask == 0xa6 || mask == 0x4e || mask == 0xce|| mask == 0x76 || mask == 0xf6;

        // If we're at the beginning of the line, set the left bit to 0. Otherwise,
        // use bit 0 of our first byte
        let left_bit = if beginning { 0 } else { bit(memory[i - 1], 6) };
        // If we're at the end of the line, set the right bit to 0. Otherwise,
        // use bit 0 of the byte after this pair
        let right_bit = if end || i + 2 >= memory.len() { 0 } else { bit(memory[i + 2], 0) };
        let colors
            = calculate_correct_colors_from_bytes(left_bit, memory[i], memory[i + 1], right_bit);
        let (mut x, y) = screen.calculate_coordinates(i);
        for color in colors {
            result.push(Pixel::new(x, y, color));
            x += 1;
        }
    }
    result
}

pub fn rect_filled(ui: &mut Ui, x1: f32, y1: f32, x2: f32, y2: f32, color: Color32) {
    ui.painter().rect_filled(Rect::from_points(&[Pos2::new(x1, y1), Pos2::new(x2, y2)]),
        0.0,
        color);
}

/// Return the correct color of the given bit triplet
fn correct_color(high_bit: u8, even: bool, previous: u8, current: u8, next: u8) -> AColor {
    use AColor::*;
    let result = match (previous, current, next) {
        (0, 0, _) | (_, 0, 0) => { Black }
        (1, 1, _) | (_, 1, 1) => { White }
        bits => {
            if (even && bits == (0, 1, 0)) || (! even && bits == (1, 0, 1)) {
                if high_bit == 1 { Blue } else { Magenta }
            } else if (even && bits == (1, 0, 1)) || (! even && bits == (0, 1, 0)) {
                if high_bit == 1 { Orange } else { Green }
            } else {
                panic!("Should never happen");
            }
        }
    };

    result
}
