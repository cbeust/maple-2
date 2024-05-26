use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use eframe::egui::Color32;
use crate::ui::hires_screen::AColor::{DHAqua, DHBlack, DHBrown, DHDarkBlue, DHDarkGreen, DHDarkRed, DHGray, DHGrey, DHLightBlue, DHLightGreen, DHMediumBlue, DHOrange, DHPink, DHPurple, DHWhite, DHYellow};

/// Device agnostic representation of a high resolution graphics for the Apple ][
/// `calculate_pixels()` returns a vector of pixels which can then be actually displayed
pub struct HiresScreen {
    line_map: HashMap<u16, u16>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AColor {
    /// Hires color
    Black, White, Green, Orange, Magenta, Blue,
    
    /// Double hires colors
    DHBlack, DHDarkRed, DHBrown, DHOrange, DHDarkGreen, DHGray, DHLightGreen, DHYellow,
    DHDarkBlue, DHPurple, DHGrey, DHPink, DHMediumBlue, DHLightBlue, DHAqua, DHWhite
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

impl Into<Color32> for AColor {
    fn into(self) -> Color32 {
        let (r, g, b) = self.to_rgb();
        Color32::from_rgb(r, g, b)
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
}
