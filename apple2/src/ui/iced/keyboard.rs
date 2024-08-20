use iced::advanced::graphics::core::SmolStr;
use iced::keyboard::{Key};
use iced::keyboard::key::Named;
use crate::ui::iced::message::{InternalUiMessage, SpecialKeyMsg};
use crate::ui::iced::message::InternalUiMessage::SpecialKey;

pub fn handle_keyboard(key: Key, modifiers: iced::keyboard::Modifiers)
    -> Option<InternalUiMessage>
{
    let mut result: Option<InternalUiMessage> = None;
    match key {
        Key::Named(k) => {
            result = named_key(k).map(InternalUiMessage::Key);
        }
        Key::Character(c) => {
            result = character_key(c, modifiers.shift(), modifiers.control())
                .map(InternalUiMessage::Key);
        }
        Key::Unidentified => {
            println!("Unidentified: {key:#?}")
        }
    }

    result
}

fn character_key(s: SmolStr, shift: bool, control: bool) -> Option<u8> {
    let c = s.chars().next().unwrap();
    let n = c as u8;

    fn k(control: bool, shift: bool, v1: u8, v2: u8, v3: u8) -> Option<u8> {
        Some(if control { v1 } else if shift { v2 } else { v3 } )
    }

    if (0x61..0x7a).contains(&n) {
        // a..z
        k(control, shift, n + 0x20, n + 0x60, n + 0x60)
    } else {
        match n {
            0x2d => { Some(if shift { 0xdf } else { 0xad }) } // _ -
            0x3d => { Some(if shift { 0xab } else { 0xbd }) } // = +
            0x5b => { Some(if shift { 0xfb } else { 0xdb }) } // { [
            0x5c => { Some(if shift { 0xfc } else { 0xdc }) } // | \
            0x5d => { Some(if shift { 0xfd } else { 0xdd }) } // } ]
            0x3b => { Some(if shift { 0xba } else { 0xbb }) } // : ;
            0x27 => { Some(if shift { 0xa2 } else { 0xa7 }) } // " '
            0x2c => { Some(if shift { 0xbc } else { 0xac }) } // < ,
            0x2e => { Some(if shift { 0xbe } else { 0xae }) } // > .
            0x2f => { Some(if shift { 0xbf } else { 0xaf }) } // ? /

            // Numbers
            0x30 => { Some(if shift { 0xa9 } else { n + 0x80 }) } // 0 )
            0x31 => { Some(if shift { 0xa1 } else { n + 0x80 }) } // 1 !
            0x32 => { Some(if shift { 0x80 } else { n + 0x80 }) } // 2 @
            0x33 => { Some(if shift { 0xa3 } else { n + 0x80 }) } // 3 #
            0x34 => { Some(if shift { 0xa4 } else { n + 0x80 }) } // 4 $
            0x35 => { Some(if shift { 0xa5 } else { n + 0x80 }) } // 5 %
            0x36 => { Some(if shift { 0xa6 } else { n + 0x80 }) } // 6 &
            0x37 => { Some(if shift { 0xa7 } else { n + 0x80 }) } // 7 '
            0x38 => { Some(if shift { 0xaa } else { n + 0x80 }) } // 8 *
            0x39 => { Some(if shift { 0xa8 } else { n + 0x80 }) } // 9 (
            _ => { None }
        }
    }
}

pub fn special_named_key(named: Named) -> Option<SpecialKeyMsg> {
    match named {
        Named::Alt => { Some(SpecialKeyMsg::AltLeft) }
        Named::AltGraph => { Some(SpecialKeyMsg::AltRight) }
        _ => { None }
    }
}

fn named_key(named: Named) -> Option<u8> {
    match named {
        Named::Enter => { Some(0x8d) }
        Named::Tab => { Some(0x89) }
        Named::Space => { Some(0xa0) }
        Named::ArrowDown => { Some(0x8a) }
        Named::ArrowLeft => { Some(0x88) }
        Named::ArrowRight => { Some(0x95) }
        Named::ArrowUp => { Some(0x8b) }
        Named::Backspace => { Some(0x88) }
        Named::Escape => { Some(0x9b) }
        _ => { None }
    }
}