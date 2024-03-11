//! Norwegian keyboard support
// from https://github.com/rust-embedded-community/pc-keyboard/blob/9acb8cacf10ffb7796c06ee1dd16f73755fd85ba/src/layouts/no105.rs
// Copyright (c) 2020 Rust Embedded Community Developers

// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:

// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use pc_keyboard::{DecodedKey, HandleControl, KeyCode, KeyboardLayout, Modifiers};

/// A standard Norwegian 102-key (or 105-key including Windows keys) keyboard.
///
/// Has a 2-row high Enter key, with Oem5 next to the left shift (ISO format).
pub struct No105Key;

trait ModifiersExt {
    fn is_altgr(&self) -> bool;
}

impl ModifiersExt for Modifiers {
    fn is_altgr(&self) -> bool {
        self.alt_gr
    }
}

impl KeyboardLayout for No105Key {
    fn map_keycode(
        &self,
        keycode: KeyCode,
        modifiers: &Modifiers,
        handle_ctrl: HandleControl,
    ) -> DecodedKey {
        let map_to_unicode = handle_ctrl == HandleControl::MapLettersToUnicode;
        match keycode {
            KeyCode::Escape => DecodedKey::Unicode(0x1B.into()),
            KeyCode::Oem8 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('§')
                } else {
                    DecodedKey::Unicode('|')
                }
            }
            KeyCode::Key1 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('!')
                } else {
                    DecodedKey::Unicode('1')
                }
            }
            KeyCode::Key2 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('"')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('@')
                } else {
                    DecodedKey::Unicode('2')
                }
            }
            KeyCode::Key3 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('#')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('£')
                } else {
                    DecodedKey::Unicode('3')
                }
            }
            KeyCode::Key4 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('¤')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('$')
                } else {
                    DecodedKey::Unicode('4')
                }
            }
            KeyCode::Key5 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('%')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('€')
                } else {
                    DecodedKey::Unicode('5')
                }
            }
            KeyCode::Key6 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('&')
                } else {
                    DecodedKey::Unicode('6')
                }
            }
            KeyCode::Key7 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('/')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('{')
                } else {
                    DecodedKey::Unicode('7')
                }
            }
            KeyCode::Key8 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('(')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('[')
                } else {
                    DecodedKey::Unicode('8')
                }
            }
            KeyCode::Key9 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode(')')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode(']')
                } else {
                    DecodedKey::Unicode('9')
                }
            }
            KeyCode::Key0 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('=')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('}')
                } else {
                    DecodedKey::Unicode('0')
                }
            }
            KeyCode::OemMinus => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('?')
                } else {
                    DecodedKey::Unicode('+')
                }
            }
            KeyCode::OemPlus => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('`')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('´')
                } else {
                    DecodedKey::Unicode('\\')
                }
            }
            KeyCode::Backspace => DecodedKey::Unicode(0x08.into()),
            KeyCode::Tab => DecodedKey::Unicode(0x09.into()),
            KeyCode::E => {
                if map_to_unicode && modifiers.is_ctrl() {
                    DecodedKey::Unicode('\u{0005}')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('€')
                } else if modifiers.is_caps() {
                    DecodedKey::Unicode('E')
                } else {
                    DecodedKey::Unicode('e')
                }
            }
            KeyCode::Oem4 => {
                if modifiers.is_caps() {
                    DecodedKey::Unicode('Å')
                } else {
                    DecodedKey::Unicode('å')
                }
            }
            KeyCode::Oem6 => {
                if modifiers.is_altgr() {
                    DecodedKey::Unicode('~')
                } else if modifiers.is_shifted() {
                    DecodedKey::Unicode('^')
                } else {
                    DecodedKey::Unicode('¨')
                }
            }
            KeyCode::Return => DecodedKey::Unicode(10.into()),
            KeyCode::Oem7 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('*')
                } else {
                    DecodedKey::Unicode('\'')
                }
            }
            KeyCode::Oem1 => {
                if modifiers.is_caps() {
                    DecodedKey::Unicode('Ø')
                } else {
                    DecodedKey::Unicode('ø')
                }
            }
            KeyCode::Oem3 => {
                if modifiers.is_caps() {
                    DecodedKey::Unicode('Æ')
                } else {
                    DecodedKey::Unicode('æ')
                }
            }
            KeyCode::M => {
                if map_to_unicode && modifiers.is_ctrl() {
                    DecodedKey::Unicode('\u{000D}')
                } else if modifiers.is_altgr() {
                    DecodedKey::Unicode('µ')
                } else if modifiers.is_caps() {
                    DecodedKey::Unicode('M')
                } else {
                    DecodedKey::Unicode('m')
                }
            }
            KeyCode::OemComma => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode(';')
                } else {
                    DecodedKey::Unicode(',')
                }
            }
            KeyCode::OemPeriod => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode(':')
                } else {
                    DecodedKey::Unicode('.')
                }
            }
            KeyCode::Oem2 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('_')
                } else {
                    DecodedKey::Unicode('-')
                }
            }
            KeyCode::Oem5 => {
                if modifiers.is_shifted() {
                    DecodedKey::Unicode('>')
                } else {
                    DecodedKey::Unicode('<')
                }
            }
            KeyCode::NumpadPeriod => {
                if modifiers.numlock {
                    DecodedKey::Unicode(',')
                } else {
                    DecodedKey::Unicode(127.into())
                }
            }
            e => {
                let us = pc_keyboard::layouts::Us104Key;
                us.map_keycode(e, modifiers, handle_ctrl)
            }
        }
    }
}