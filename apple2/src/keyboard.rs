use eframe::egui::{Key, Modifiers};
use eframe::egui::Key::*;

pub(crate) fn key_to_char(key: &Key, modifiers: &Modifiers) -> u8 {
    let result = if modifiers.shift {
        match key {
            Num0 => { 0xa9 }
            Num1 => { 0xa1 }
            Num2 => { 0xa2 }
            Num3 => { 0xa3 }
            Num4 => { 0xa4 }
            Num5 => { 0xa5 }
            Num6 => { 0xbf }
            Num7 => { 0xa6 }
            Num8 => { 0xa7 }
            Num9 => { 0xa8 }
            Minus => { 0xdf }

            _ => { k(key) }
        }
    } else if modifiers.ctrl {
        match key {
            A => { 0x81 }
            B => { 0x82 }
            C => { 0x83 }
            D => { 0x84 }
            E => { 0x85 }
            F => { 0x86 }
            G => { 0x87 }
            H => { 0x88 }
            I => { 0x89 }
            J => { 0x8a }
            K => { 0x8b }
            L => { 0x8c }
            M => { 0x8d }
            N => { 0x8e }
            O => { 0x8f }
            P => { 0x90 }
            Q => { 0x91 }
            R => { 0x92 }
            S => { 0x93 }
            T => { 0x94 }
            U => { 0x95 }
            V => { 0x96 }
            W => { 0x97 }
            X => { 0x98 }
            Y => { 0x99 }
            Z => { 0x9a }
            _ => { k(key) }
        }
    } else {
        k(key)
    };

    result
}

fn k(key: &Key) -> u8 {
    match key {
        ArrowLeft | Backspace => { 0x88 }
        ArrowRight => { 0x95 }
        ArrowDown => { 0x8a }
        ArrowUp => { 0x8b }
        Escape => { 0x9b }
        Tab => { 0x89 }
        Enter => { 0x8d }
        Space => { 0xa0 }
        Colon => { 0xba }
        Comma => { 0xac }
        Backslash => { 0xdc }
        OpenBracket => { 0xdb }
        CloseBracket => { 0xdd }
        Minus => { 0xad }
        Period => { 0xae }
        Plus => { 0xab }
        Equals => { 0xbd }
        Semicolon => { 0xbb }
        Num0 => { 0xb0 }
        Num1 => { 0xb1 }
        Num2 => { 0xb2 }
        Num3 => { 0xb3 }
        Num4 => { 0xb4 }
        Num5 => { 0xb5 }
        Num6 => { 0xb6 }
        Num7 => { 0xb7 }
        Num8 => { 0xb8 }
        Num9 => { 0xb9 }
        Delete => { 0xff }
        A => { 0xc1 }
        B => { 0xc2 }
        C => { 0xc3 }
        D => { 0xc4 }
        E => { 0xc5 }
        F => { 0xc6 }
        G => { 0xc7 }
        H => { 0xc8 }
        I => { 0xc9 }
        J => { 0xca }
        K => { 0xcb }
        L => { 0xcc }
        M => { 0xcd }
        N => { 0xce }
        O => { 0xcf }
        P => { 0xd0 }
        Q => { 0xd1 }
        R => { 0xd2 }
        S => { 0xd3 }
        T => { 0xd4 }
        U => { 0xd5 }
        V => { 0xd6 }
        W => { 0xd7 }
        X => { 0xd8 }
        Y => { 0xd9 }
        Z => { 0xda }

        _ => { 0xbf }
    }
}