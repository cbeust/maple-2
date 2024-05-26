use crate::misc::bit;
use crate::ui::graphics_screen::calculate_correct_colors_from_bytes;
use crate::ui::hires_screen::{AColor, CONSECUTIVES, HiresScreen, INTERLEAVING};
use crate::ui::hires_screen::AColor::{Green, Magenta};

#[test]
pub fn test_calculate_coordinates_double_hires() {
    let data = [
        (0x0000, (0, 0)),
        (0x0002, (14, 0)),
        (0x0026, (266, 0)),
        (0x0028, (280, 0)),
        (0x004e, (546, 0)),
        (0x0050, (560, 0)),
        (0x0400, (0, 1)),
    ];

    for (address, (expected_x, expected_y)) in data {
        let (x, y) = calculate_coordinates_double_hires(address);
        assert_eq!(x, expected_x, "Incorrect X received");
        assert_eq!(y, expected_y, "Incorrect X received");
    }
}

#[test]
pub fn test_coordinates_for_hires() {
    let expected = [
        (0, (0, 0)),
        (2, (7, 0)),
        (4, (14, 0)),
        (8, (28, 0)),
        (0x10, (56, 0)),
        (0x20, (112, 0)),
        (0x21, (119, 0)),
        (0x23, (126, 0)),
        (0x26, (140, 0)),
        // (1, (4, 0)),
        // (0x14a8, (0, 77)),
        // (0x14a9, (4, 77)),
        // (0x1f77, (156, 183)),
    ];
    for (address, (x, y)) in expected {
        let (xx, yy) = calculate_coordinates_double_hires(address);
        assert_eq!(xx, x, "Bad X coordinate for {:04X}", address);
        assert_eq!(yy, y, "Bad Y coordinate for {:04X}", address);
    }
}

#[test]
pub fn test_correct_colors_from_bytes() {
    use crate::ui::hires_screen::AColor::*;
    let data = [
        ([0x2a, 0x55], Green),
        ([0x55, 0x2a], Magenta),
    ];
    for (bytes, expected_color) in data {
        let colors = calculate_correct_colors_from_bytes(0, bytes[0], bytes[1], 0);
        assert_eq!(colors.len(), 14);
        for i in 1..colors.len() - 1 {
            assert_eq!(colors[i], expected_color);
            // println!("  Color: {:#?}", colors[i]);
        }
    }
}


#[test]
pub fn test_coordinates() {
    let data = [
        (0x0, (0, 0)),
        (0x200, (0, 32))
    ];

    let screen = HiresScreen::new();
    for (address, (x, y)) in data {
        assert_eq!(screen.calculate_coordinates(address), (x, y));
    }
}

#[cfg(test)]
pub fn calculate_coordinates_double_hires(a: usize /* 0-0x2000 */) -> (u16, u16) {
    assert!((0..0x2000).contains(&a));
    assert_eq!(a % 2, 0);

    let mut y = 0;
    for i in INTERLEAVING {
        for c in CONSECUTIVES {
            let r = c + i;
            if (r..r + 0x28).contains(&(a as u16)) {
                let x = (a as u16 - r) * 7;
                // println!("r: {r:04X }Address: {:04X} -> {x}, {y}", a);
                return (x, y);
            }
            y += 1;
        }
    }
    panic!("Can't calculate coordinates for {a:04X}");
}

