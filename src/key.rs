//! This module contains functions for controlling the keyboard.
extern crate rand;

#[cfg(target_os = "macos")]
use core_graphics::event;
#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode,
                           EventField};
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSourceStateID::HIDSystemState;

use self::rand::Rng;
use std::{thread, time};

/// Device-independent modifier flags.
#[derive(Copy, Clone, Debug)]
pub enum Flag {
    Shift,
    Control,
    Alt,
    Meta,

    // Special key identifiers.
    Help,
    SecondaryFn,

    /// Identifies key events from numeric keypad area on extended keyboards.
    NumericPad,

    /// Indicates if mouse/pen movement events are not being coalesced.
    NonCoalesced,
}

/// Device-independent key codes.
#[derive(Copy, Clone, Debug)]
pub enum KeyCode {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    LeftArrow,
    Control,
    RightArrow,
    DownArrow,
    End,
    UpArrow,
    PageUp,
    Alt,
    Return,
    PageDown,
    Delete,
    Home,
    Escape,
    Backspace,
    Meta,
    CapsLock,
    Shift,
}

pub trait KeyCodeConvertible {
    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode;
    fn character(&self) -> Option<char> {
        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Character(pub char);

#[derive(Copy, Clone, Debug)]
pub struct Code(pub KeyCode);

/// Attempts to simulate typing a string at the given WPM, or as fast as
/// possible if the WPM is 0.
pub fn type_string(string: &str, wpm: Option<f64>, noise: Option<f64>, flags: &[Flag]) {
    let wpm = wpm.unwrap_or(0.0);
    let noise = noise.unwrap_or(0.0);
    let cpm = wpm * 5.0;
    let cps = cpm / 60.0;
    let ms_per_character: u64 = if cps == 0.0 {
        0
    } else {
        (1000.0 / cps).round() as u64
    };

    for c in string.chars() {
        let tolerance = (noise * ms_per_character as f64).round() as u64;
        let noise = if tolerance > 0 {
            rand::thread_rng().gen_range(0, tolerance)
        } else {
            0
        };

        tap(Character(c), flags);
        thread::sleep(time::Duration::from_millis(ms_per_character + noise));
    }
}

/// Convenience wrapper around `toggle()` that holds down and then releases the
/// given key and modifier flags.
pub fn tap<T: KeyCodeConvertible + Copy>(key: T, flags: &[Flag]) {
    let ms: u64 = rand::thread_rng().gen_range(10, 20);
    toggle(key, true, flags);
    thread::sleep(time::Duration::from_millis(ms));
    toggle(key, false, flags);
}

/// Holds down the given key or keycode if `down` is `true`, or releases it if
/// not. Characters are converted to a keycode corresponding to the current
/// keyboard layout.
pub fn toggle<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    if cfg!(target_os = "macos") {
        toggle_macos(key, down, flags);
    } else {
        panic!("Unsupported OS");
    }
}

#[cfg(target_os = "macos")]
fn toggle_macos<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    use core_graphics::event::CGEventType::*;
    let source = CGEventSource::new(HIDSystemState).unwrap();

    if flags.len() == 0 {
        if let Some(character) = key.character() {
            let mut buf = [0; 2];
            let event = CGEvent::new_keyboard_event(source, 0, down).unwrap();
            event.set_string_from_utf16_unchecked(character.encode_utf16(&mut buf));
            event.post(CGEventTapLocation::HID);
            return;
        }
    }

    let event = CGEvent::new_keyboard_event(source, key.code(), down).unwrap();
    let event_type: CGEventType = if down { KeyDown } else { KeyUp };
    event.set_type(event_type);
    event.set_flags(event_mask_for_flags(flags));
    event.post(CGEventTapLocation::HID);
}

#[cfg(target_os = "macos")]
fn char_to_key_code(character: char) -> CGKeyCode {
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event = CGEvent::new_keyboard_event(source, 0, true).unwrap();
    let mut buf = [0; 2];
    event.set_string_from_utf16_unchecked(character.encode_utf16(&mut buf));
    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as CGKeyCode
}

impl KeyCodeConvertible for Character {
    fn character(&self) -> Option<char> {
        Some(self.0)
    }

    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode {
        char_to_key_code(self.0)
    }
}

impl KeyCodeConvertible for Code {
    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode {
        CGKeyCode::from(self.0)
    }
}

#[cfg(target_os = "macos")]
impl From<Flag> for CGEventFlags {
    fn from(flag: Flag) -> CGEventFlags {
        match flag {
            Flag::Shift => event::CGEventFlags::CGEventFlagShift,
            Flag::Control => event::CGEventFlags::CGEventFlagControl,
            Flag::Alt => event::CGEventFlags::CGEventFlagAlternate,
            Flag::Meta => event::CGEventFlags::CGEventFlagCommand,
            Flag::Help => event::CGEventFlags::CGEventFlagHelp,
            Flag::SecondaryFn => event::CGEventFlags::CGEventFlagSecondaryFn,
            Flag::NumericPad => event::CGEventFlags::CGEventFlagNumericPad,
            Flag::NonCoalesced => event::CGEventFlags::CGEventFlagNonCoalesced,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<KeyCode> for CGKeyCode {
    fn from(code: KeyCode) -> CGKeyCode {
        match code {
            KeyCode::F1 => event::KeyCode::F1,
            KeyCode::F2 => event::KeyCode::F2,
            KeyCode::F3 => event::KeyCode::F3,
            KeyCode::F4 => event::KeyCode::F4,
            KeyCode::F5 => event::KeyCode::F5,
            KeyCode::F6 => event::KeyCode::F6,
            KeyCode::F7 => event::KeyCode::F7,
            KeyCode::F8 => event::KeyCode::F8,
            KeyCode::F9 => event::KeyCode::F9,
            KeyCode::F10 => event::KeyCode::F10,
            KeyCode::F11 => event::KeyCode::F11,
            KeyCode::F12 => event::KeyCode::F12,
            KeyCode::LeftArrow => event::KeyCode::LEFT_ARROW,
            KeyCode::Control => event::KeyCode::CONTROL,
            KeyCode::RightArrow => event::KeyCode::RIGHT_ARROW,
            KeyCode::DownArrow => event::KeyCode::DOWN_ARROW,
            KeyCode::End => event::KeyCode::END,
            KeyCode::UpArrow => event::KeyCode::UP_ARROW,
            KeyCode::PageUp => event::KeyCode::PAGE_UP,
            KeyCode::Alt => event::KeyCode::OPTION,
            KeyCode::Return => event::KeyCode::RETURN,
            KeyCode::PageDown => event::KeyCode::PAGE_DOWN,
            KeyCode::Delete => event::KeyCode::DELETE,
            KeyCode::Home => event::KeyCode::HOME,
            KeyCode::Escape => event::KeyCode::ESCAPE,
            KeyCode::Backspace => event::KeyCode::DELETE,
            KeyCode::Meta => event::KeyCode::COMMAND,
            KeyCode::CapsLock => event::KeyCode::CAPS_LOCK,
            KeyCode::Shift => event::KeyCode::SHIFT,
        }
    }
}

#[cfg(target_os = "macos")]
fn event_mask_for_flags(flags: &[Flag]) -> CGEventFlags {
    let map = flags.iter().map(|&x| CGEventFlags::from(x));
    map.fold(event::CGEventFlags::CGEventFlagNull, |x, y| {
        x | y as CGEventFlags
    })
}
