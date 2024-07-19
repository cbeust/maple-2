use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::constants::*;
use crate::messages::DrawCommand;
use crate::messages::DrawCommand::Rectangle;
use crate::misc::bit;
use crate::{soft_switch};
use crate::memory_constants::*;
use crate::ui::hires_screen::AColor::*;

/// Device-agnostic representation of a high resolution graphics for the Apple ][
/// `calculate_pixels()` returns a vector of pixels which can then be actually displayed
#[derive(Default)]
pub struct HiresScreen {
    line_map: HashMap<u16, u16>,

    /// Double hires graphics
    /// RGB mode (0-3)
    pub(crate) dhg_rgb_mode: u8,

    /// Used in double hires to keep track of the color of the last cell
    pub(crate) last_cell_is_color: bool,
    /// Used in double hires to keep track of the last bit written
    pub(crate) last_bit: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AColor {
    /// Hires color
    Black, White, Green, Orange, Magenta, Blue,
    
    /// Double hires colors
    DHBlack, DHDarkRed, DHBrown, DHOrange, DHDarkGreen, DHGray, DHLightGreen, DHYellow,
    DHDarkBlue, DHPurple, DHGrey, DHPink, DHMediumBlue, DHLightBlue, DHAqua, DHWhite
}

impl Hash for AColor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_rgb().hash(state);
    }
}

impl AColor {
    /// https://mrob.com/pub/xapple2/colors.html
    pub(crate) fn to_rgb(&self) -> (u8, u8, u8) {
        use AColor::*;

        match self {
            Black => { (0, 0, 0) }
            White => { (0xff, 0xff, 0xff) }
            Green => { (20, 245, 60) }
            Orange => { (255, 106, 60) }
            Magenta => { (255, 68, 253) }
            Blue => { (20, 207, 254) }

            DHBlack      => { (0x00, 0x00, 0x00) }
            DHDarkRed    => { (0x9d, 0x09, 0x66) }
            DHDarkBlue   => { (0x2A, 0x2A, 0xE5) }
            DHPurple     => { (0xc7, 0x32, 0x34) }   // magenta
            DHDarkGreen  => { (0x00, 0x80, 0x00) }
            DHGray       => { (0x80, 0x80, 0x80) }
            DHMediumBlue => { (0x0D, 0xA1, 0xFF) }
            DHLightBlue  => { (0xAA, 0xAA, 0xFF) }
            DHBrown      => { (0x55, 0x55, 0x00) }
            DHOrange     => { (0xF2, 0x5E, 0x00) }
            DHGrey       => { (0xC0, 0xC0, 0xC0) }  // light grey
            DHPink       => { (0xFF, 0x89, 0xE5) }
            DHLightGreen => { (0x38, 0xCB, 0x00) }  // green
            DHYellow     => { (0xD5, 0xD5, 0x1A) }
            DHAqua       => { (0x62, 0xF6, 0x99) }
            DHWhite      => { (0xFF, 0xFF, 0xFF) }
        }
    }

    //noinspection ALL
    pub fn to_double_hires_color(color: u8) -> Self {
        match color {
            0 => { DHBlack }
            1 => { DHDarkRed }
            2 => { DHDarkBlue  }
            3 => { DHPurple }
            4 => { DHDarkGreen }
            5 => { DHGray }
            6 => { DHMediumBlue }
            7 => { DHLightBlue /* magenta */  }
            8 => { DHBrown }
            9 => { DHOrange  }
            10 => { DHGrey }
            11 => { DHPink }
            12 => { DHLightGreen }
            13 => { DHYellow }
            14 => { DHAqua }
            15 => { DHWhite }
            _ => { panic!("Should never happen"); }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Pixel {
    pub x: u16,
    pub y: u16,
    pub color: AColor,
}

impl Pixel {
    pub(crate) fn new(x: u16, y: u16, color: AColor) -> Self {
        Self { x, y, color }
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{},{}-{:?}", self.x, self.y, self.color)).unwrap();
        Ok(())
    }
}

const _INTERLEAVING_FLIPPED: [u16; 24] = [
    0x3d0, 0x350, 0x2d0, 0x250, 0x1d0, 0x150, 0xd0, 0x50,
    0x3a8, 0x328, 0x2a8, 0x228, 0x1a8, 0x128, 0xa8, 0x28,
    0x380, 0x300, 0x280, 0x200, 0x180, 0x100, 0x80, 0
];
pub const INTERLEAVING: [u16; 24] = [
    0, 0x80, 0x100, 0x180, 0x200, 0x280, 0x300, 0x380,
    0x28, 0xa8, 0x128, 0x1a8, 0x228, 0x2a8, 0x328, 0x3a8,
    0x50, 0xd0, 0x150, 0x1d0, 0x250, 0x2d0, 0x350, 0x3d0,
];

pub const CONSECUTIVES: [u16; 8] = [0, 0x400, 0x800, 0xc00, 0x1000, 0x1400, 0x1800, 0x1c00];
const _CONSECUTIVES_FLIPPED: [u16; 8] = [0x1c00, 0x1800, 0x1400, 0x1000, 0xc00, 0x800, 0x400, 0];

#[derive(Default)]
pub struct Hgr {
    // 0..39, then increment interleaving
    x: usize,
    y: usize,
    interleave: usize,
    consecutive: usize,
    total: usize,
}

impl Iterator for Hgr {
    type Item = usize;

    /// Line iterator
    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.interleave < INTERLEAVING.len() {
            Some((INTERLEAVING[self.interleave] + CONSECUTIVES[self.consecutive]) as usize)
        } else {
            None
        };
        self.consecutive += 1;
        if self.consecutive == CONSECUTIVES.len() {
            self.consecutive = 0;
            self.interleave += 1;
        }
        result
    }
}

impl HiresScreen {
    pub fn new() -> Self {
        let mut line_map: HashMap<u16, u16> = HashMap::new();
        let mut l = 0;
        for il in INTERLEAVING {
            for c in CONSECUTIVES {
                line_map.insert(il + c, l);
                l += 1;
            }
        }

        Self {
            line_map,
            dhg_rgb_mode: 0,
            last_cell_is_color: true,
            last_bit: 0,
        }
    }

    //
    // Now we have x,y and 7 pixels to write
    //
    // Finally, another quirk of Wozniak's design is that while any pixel could be black or white,
    // only pixels with odd X-coordinates could be green or orange. Likewise, only even-numbered pixels
    // could be violet or blue.[4]
    //
    // pub fn color(group: u8, bits: u8, _x: u16) -> AColor {
    //     use AColor::*;
    //     match bits {
    //         0 => { Black }
    //         3 => { White }
    //         2 => { /* if x%2 == 0 {Black} else */ if group == 0 {Green} else {Orange} }
    //         _ => { /* if x%2 == 1 Black else */ if group == 0 {Magenta} else {Blue} }
    //     }
    // }

    /// The location must be 0..<0x2000
    pub fn calculate_coordinates(&self, location: usize) -> (u16, u16) {
        let even = location % 2 == 0;
        //
        // Calculate x,y
        //
        if location >= 0x2000 {
            panic!("ERROR COORDINATES");
        }
        let even_location = if even { location } else { location - 1 };
        let loc = even_location as i16;
        let mut closest = i16::MAX;
        let mut key = 0_u16;
        for k in self.line_map.keys() {
            let distance: i16 = loc - *k as i16;
            if 0 <= distance && distance <= closest {
                closest = distance;
                key = *k;
            }
        }
        let y = self.line_map.get(&key).unwrap();
        let x = ((loc - key as i16) * 7) as u16;
        // println!("Coordinates for {location:04X}: {x},{y}");

        (x, *y)
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

    pub fn calculate_hires(&mut self, memory: &[u8], height: u16, mag: u16, page2: bool) -> Vec<DrawCommand> {
        let mag = mag as f32;
        let memory = if page2 { &memory[0x4000..0x6000] }
        else { &memory[0x2000..0x4000] };
        let pixels = HiresScreen::calculate_pixels(memory);
        let mut result: Vec<DrawCommand> = Vec::new();
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

    pub(crate) fn calculate_text(&mut self,
            memory: &Vec<u8>, aux_memory: &Vec<u8>,
            mag: u16, is_80: bool, mixed: bool, page2: bool)
            -> Vec<DrawCommand>
    {
        let mut result: Vec<DrawCommand> = Vec::new();
        let start = if mixed { TEXT_HEIGHT - 4 } else { 0 };
        for y in start..TEXT_HEIGHT {
            for x in 0..if is_80 { TEXT_WIDTH * 2 } else { TEXT_WIDTH } {
                let address: usize;
                let c_a = if is_80 {
                    let actual_x = x / 2;
                    address = text_coordinates_to_address(actual_x, y, page2) as usize;
                    if x % 2 == 0 {
                        aux_memory[address]
                    } else {
                        memory[address]
                    }
                } else {
                    address = text_coordinates_to_address(x, y, page2) as usize;
                    memory[address]
                };
                // if alt_charset {
                //     c_a = c_a & 0x3f;
                // }
                let mag_width = if is_80 { mag / 2 } else { mag };
                for yy in 0..FONT_WIDTH {
                    let bits = crate::ui::text_screen::TEXT_ROM[yy as usize + ((c_a as usize) << 3)];
                    // println!("Drawing byte {:02X}", c_a);
                    for xx in 0..FONT_HEIGHT {
                        let pixel = (bits >> xx) & 1;
                        let color = if pixel == 0 { White } else { Black };
                        let x0: u16 = (x as u16 * 7 + xx as u16) * mag_width;
                        let y0: u16 = (y as u16 * 8 + yy as u16) * mag;
                        let x1 = x0 + mag;
                        let y1 = y0 + mag;

                        // println!("Drawing {},{} - {},{} color: {:#?}", x0, y0, x1, y1, color);
                        result.push(Rectangle(x0 as f32, y0 as f32,
                            x1 as f32, y1 as f32, color));
                    }
                }
                // println!("Done with {},{}", x, y);
            }
        }

        result
    }

    pub fn calculate_double_hires2(&mut self,
            memory: &Vec<u8>, aux_memory: &Vec<u8>,
            mag: u16, page2: bool) -> Vec<DrawCommand> {
        let mut result: Vec<DrawCommand> = Vec::new();
        let hgr = Hgr::default();
        let start = if page2 { 0x4000 } else { 0x2000 };

        for (y, a) in hgr.enumerate() {
            let mut x = 0;
            let mut address = a;
            while x < 560 {
                let commands = self.get_double_hires_draw_commands(memory, aux_memory,
                    mag, x as u16, y as u16, start + address);
                x += commands.len();
                result.extend(commands);
                address += 1;
            }
        }

        result
    }

    /// Shamelessly copied from AppleWin's implementation
    fn get_double_hires_draw_commands(&mut self,
            memory: &Vec<u8>, aux_memory: &Vec<u8>,
            mag: u16, x: u16, y: u16, mut address: usize)
            -> Vec<DrawCommand> {
        // "Mixed mode" is really black and white mode, only happens for a combination
        // of the F1 and F2 switches "10" == 2. See [crate::memory::update_f1_f2()] for details.
        let is_mixed_mode = self.dhg_rgb_mode == 2;
        let mut result: Vec<DrawCommand> = Vec::new();
        let x_offset = address & 1;
        address -= x_offset;

        let byteval1 = aux_memory[address] as u32;
        let byteval2 = memory[address] as u32;
        let byteval3 = aux_memory[address + 1] as u32;
        let byteval4 = memory[address + 1] as u32;
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
            let mag2 = mag / 2;
            let x0 = (x_index + x as usize) * mag2 as usize;
            x_index += 1;
            let x1 = x0 + mag2 as usize;
            let y0 = y as usize * mag as usize;
            let y1 = y0 + mag as usize;

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

    pub fn on_reboot(&mut self) {
        self.dhg_rgb_mode = 0;
    }

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

    pub fn calculate_double_hires(&self, memory: &Vec<u8>, aux_memory: &Vec<u8>, page2: bool)
        -> Vec<DrawCommand>
    {
        let mut result: Vec<DrawCommand> = Vec::new();
        let start = if page2 { 0x4000 } else { 0x2000 };

        // Calculate pixel colors by mixing aux and main memory bytes
        let mut x = 0;
        let hgr = Hgr::default();
        for (y, a) in hgr.enumerate() {
            for byte_index in (0..0x28).step_by(2) {
                let a2 = start + a + byte_index;
                let mask = a2 & 0xff;
                let byte0 = aux_memory[a2];
                let byte1 = memory[a2];
                let byte2 = aux_memory[a2 + 1];
                let byte3 = memory[a2 + 1];

                // Beginning of line?
                let beginning = [0, 0x80, 0x28, 0xa8, 0x50, 0x50].contains(&mask);
                // End of line? (-1 the actual end of line since we look at bytes as pairs)
                let end = [0x26, 0xa6, 0x4e, 0xce, 0x76, 0xf6].contains(&mask);

                let left_byte = if beginning { 0 } else { memory[a2 - 1] };
                let right_byte = if end { 0 } else { aux_memory[a2 + 2] };

                let bits = HiresScreen::bits(left_byte, byte0, byte1, byte2, byte3, right_byte);
                let colors = HiresScreen::calculate_double_hires_colors_from_bits(bits);

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

    pub fn get_draw_commands(&mut self, memory: &Vec<u8>, aux_memory:&Vec<u8>,
        mag: u16) -> Vec<DrawCommand>
    {

        if memory.is_empty() {
            return Vec::new();
        }

        // DHGR is set by writing in no particular order:
        // C0..:
        // 5E (AN3)
        // 0d (80 VID)
        // 50 (graphics)
        // 52 (full screen)
        // 54 (page1)
        // 57 (hires)
        let is_dhgr_on = || {
            let an3 = memory[AN3_STATUS as usize] & 0b0010_0000 != 0;
            let eighty = soft_switch(memory, EIGHTY_COLUMNS_STATUS);
            let is_graphics = ! soft_switch(memory, TEXT_STATUS);
            let full_screen = ! soft_switch(memory, MIXED_STATUS);
            let is_hires = soft_switch(memory, HIRES_STATUS);

            an3 && eighty && is_graphics && full_screen && is_graphics && is_hires
        };

        //
        // Retrieve the DrawCommands based on the various display modes
        //
        let mut draw_commands: Vec<DrawCommand> = Vec::new();
        if is_dhgr_on() {
            draw_commands = self.calculate_double_hires2(memory, aux_memory, mag, false /* page1 */);
        } else {
            let page2 = soft_switch(memory, PAGE_2_STATUS);
            let is_text = soft_switch(memory, TEXT_STATUS);
            let is_hires = soft_switch(memory, HIRES_STATUS);
            let is_mixed = soft_switch(memory, MIXED_STATUS);
            let is_80 = soft_switch(memory, EIGHTY_COLUMNS_STATUS);
            // let alt_charset = soft_switch(memory, ALT_CHAR_STATUS);
            if is_text {
                draw_commands = self.calculate_text(memory, aux_memory, mag, is_80,
                    false /* not mixed */, page2);
            } else if is_hires {
                if is_mixed {
                    draw_commands = self.calculate_hires(memory, HIRES_HEIGHT_MIXED, mag, page2);
                    draw_commands.append(&mut self.calculate_text(memory, aux_memory,
                        mag, is_80,
                        true /* mixed */, page2));
                } else {
                    draw_commands = self.calculate_hires(memory,
                        HIRES_HEIGHT, mag, page2);
                }
            }
        }

        draw_commands
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

pub fn text_coordinates_to_address(x: u8, y: u8, page2: bool) -> u16 {
    let mut result = TEXT_MODE_ADDRESSES[y as usize] + x as u16;
    if page2 { result += 0x400 };
    result
}
