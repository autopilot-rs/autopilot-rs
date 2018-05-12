//! This module contains functions for controlling the keyboard.
extern crate rand;

#[cfg(target_os = "macos")]
use core_graphics::event;
#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventFlags, CGKeyCode};
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSourceStateID::HIDSystemState;
#[cfg(target_os = "linux")]
use internal;
#[cfg(target_os = "linux")]
use x11;

use self::rand::Rng;
use std;

/// Device-independent modifier flags.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Flag {
    Shift,
    Control,
    Alt,
    Meta,

    // Special key identifiers.
    Help,
}

/// Device-independent key codes.
#[derive(Copy, Clone, Debug, PartialEq)]
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
    Tab,
}

pub trait KeyCodeConvertible {
    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode;
    #[cfg(target_os = "linux")]
    fn code(&self) -> XKeyCode;
    #[cfg(windows)]
    fn code(&self) -> WinKeyCode;
    fn character(&self) -> Option<char> {
        None
    }
    fn flags(&self) -> &[Flag] {
        &[]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Character(pub char);

#[derive(Copy, Clone, Debug)]
pub struct Code(pub KeyCode);

/// Attempts to simulate typing a string at the given WPM, or as fast as
/// possible if the WPM is 0.
pub fn type_string(string: &str, wpm: f64, noise: f64, flags: &[Flag]) {
    let cpm = wpm * 5.0;
    let cps = cpm / 60.0;
    let ms_per_character: u64 = if cps == 0.0 {
        0
    } else {
        (1000.0 / cps).round() as u64
    };
    let ms_per_stroke = (ms_per_character as f64 / 2.0).round() as u64;

    for c in string.chars() {
        let tolerance = (noise * ms_per_character as f64).round() as u64;
        let noise = if tolerance > 0 {
            rand::thread_rng().gen_range(0, tolerance)
        } else {
            0
        };

        tap(Character(c), ms_per_stroke, flags);
        std::thread::sleep(std::time::Duration::from_millis(ms_per_stroke + noise));
    }
}

/// Convenience wrapper around `toggle()` that holds down and then releases the
/// given key and modifier flags. Delay between pressing and releasing the key
/// can be controlled using the `delay_ms` parameter.
pub fn tap<T: KeyCodeConvertible + Copy>(key: T, delay_ms: u64, flags: &[Flag]) {
    toggle(key, true, flags);
    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    toggle(key, false, flags);
}

/// Holds down the given key or keycode if `down` is `true`, or releases it if
/// not. Characters are converted to a keycode corresponding to the current
/// keyboard layout.
pub fn toggle<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    let key_flags = key.character().map(|c| flags_for_char(c)).unwrap_or(&[]);
    let mut appended_flags: Vec<Flag> = Vec::with_capacity(flags.len() + key_flags.len());
    appended_flags.extend_from_slice(flags);
    for flag in key_flags.iter() {
        if !flags.contains(flag) {
            appended_flags.push(*flag);
        }
    }
    system_toggle(key, down, &appended_flags);
}

#[cfg(target_os = "macos")]
fn char_to_key_code(character: char) -> CGKeyCode {
    use core_graphics::event::EventField;
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event = CGEvent::new_keyboard_event(source, 0, true).unwrap();
    let mut buf = [0; 2];
    event.set_string_from_utf16_unchecked(character.encode_utf16(&mut buf));
    event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as CGKeyCode
}

#[cfg(target_os = "linux")]
fn char_to_key_code(character: char) -> XKeyCode {
    match character {
        ' ' => x11::keysym::XK_space as XKeyCode,
        '!' => x11::keysym::XK_exclam as XKeyCode,
        '#' => x11::keysym::XK_numbersign as XKeyCode,
        '$' => x11::keysym::XK_dollar as XKeyCode,
        '%' => x11::keysym::XK_percent as XKeyCode,
        '&' => x11::keysym::XK_ampersand as XKeyCode,
        '(' => x11::keysym::XK_parenleft as XKeyCode,
        ')' => x11::keysym::XK_parenright as XKeyCode,
        '*' => x11::keysym::XK_asterisk as XKeyCode,
        '+' => x11::keysym::XK_plus as XKeyCode,
        ',' => x11::keysym::XK_comma as XKeyCode,
        '-' => x11::keysym::XK_minus as XKeyCode,
        '.' => x11::keysym::XK_period as XKeyCode,
        '/' => x11::keysym::XK_slash as XKeyCode,
        ':' => x11::keysym::XK_colon as XKeyCode,
        ';' => x11::keysym::XK_semicolon as XKeyCode,
        '<' => x11::keysym::XK_less as XKeyCode,
        '=' => x11::keysym::XK_equal as XKeyCode,
        '>' => x11::keysym::XK_greater as XKeyCode,
        '?' => x11::keysym::XK_question as XKeyCode,
        '@' => x11::keysym::XK_at as XKeyCode,
        '[' => x11::keysym::XK_bracketleft as XKeyCode,
        '\'' => x11::keysym::XK_quotedbl as XKeyCode,
        '\\' => x11::keysym::XK_backslash as XKeyCode,
        ']' => x11::keysym::XK_bracketright as XKeyCode,
        '^' => x11::keysym::XK_asciicircum as XKeyCode,
        '_' => x11::keysym::XK_underscore as XKeyCode,
        '`' => x11::keysym::XK_grave as XKeyCode,
        '{' => x11::keysym::XK_braceleft as XKeyCode,
        '|' => x11::keysym::XK_bar as XKeyCode,
        '}' => x11::keysym::XK_braceright as XKeyCode,
        '~' => x11::keysym::XK_asciitilde as XKeyCode,
        '\t' => x11::keysym::XK_Tab as XKeyCode,
        '\n' => x11::keysym::XK_Return as XKeyCode,
        _ => unsafe {
            let mut buf = [0; 2];
            x11::xlib::XStringToKeysym(character.encode_utf8(&mut buf).as_ptr() as *const i8)
        },
    }
}

#[cfg(target_os = "macos")]
fn flags_for_char<'a>(_character: char) -> &'a [Flag] {
    &[]
}

#[cfg(windows)]
fn flags_for_char<'a>(_character: char) -> &'a [Flag] {
    &[]
}

#[cfg(target_os = "linux")]
fn flags_for_char<'a>(character: char) -> &'a [Flag] {
    const UPPERCASE_CHARACTERS: &[char] = &[
        '!', '#', '$', '%', '&', '(', ')', '*', '+', ':', '<', '>', '?', '@', '{', '|', '}', '~',
    ];
    if character.is_uppercase() || UPPERCASE_CHARACTERS.contains(&character) {
        &[Flag::Shift]
    } else {
        &[]
    }
}

impl KeyCodeConvertible for Character {
    fn character(&self) -> Option<char> {
        Some(self.0)
    }

    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode {
        char_to_key_code(self.0)
    }

    #[cfg(windows)]
    fn code(&self) -> WinKeyCode {
        panic!("Unsupported OS")
    }

    #[cfg(target_os = "linux")]
    fn code(&self) -> XKeyCode {
        char_to_key_code(self.0)
    }
}

impl KeyCodeConvertible for Code {
    #[cfg(target_os = "macos")]
    fn code(&self) -> CGKeyCode {
        CGKeyCode::from(self.0)
    }

    #[cfg(windows)]
    fn code(&self) -> WinKeyCode {
        WinKeyCode::from(self.0)
    }

    #[cfg(target_os = "linux")]
    fn code(&self) -> XKeyCode {
        XKeyCode::from(self.0)
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
            KeyCode::Tab => event::KeyCode::TAB,
        }
    }
}

#[cfg(target_os = "macos")]
fn cg_event_mask_for_flags(flags: &[Flag]) -> CGEventFlags {
    flags
        .iter()
        .map(|&x| CGEventFlags::from(x))
        .fold(event::CGEventFlags::CGEventFlagNull, |x, y| {
            x | y as CGEventFlags
        })
}

#[cfg(target_os = "macos")]
fn system_toggle<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    use core_graphics::event::CGEventType::*;
    use core_graphics::event::{CGEventTapLocation, CGEventType};
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
    event.set_flags(cg_event_mask_for_flags(flags));
    event.post(CGEventTapLocation::HID);
}

#[cfg(windows)]
type WinKeyCode = i32;

#[cfg(windows)]
impl From<Flag> for WinKeyCode {
    fn from(flag: Flag) -> WinKeyCode {
        use winapi::um::winuser;
        let win_code = match flag {
            Flag::Shift => winuser::VK_SHIFT,
            Flag::Control => winuser::VK_CONTROL,
            Flag::Alt => winuser::VK_MENU,
            Flag::Meta => winuser::VK_LWIN,
            Flag::Help => winuser::VK_HELP,
        };
        win_code as WinKeyCode
    }
}

#[cfg(windows)]
impl From<KeyCode> for WinKeyCode {
    fn from(code: KeyCode) -> WinKeyCode {
        use winapi::um::winuser;
        let win_code = match code {
            KeyCode::F1 => winuser::VK_F1,
            KeyCode::F2 => winuser::VK_F2,
            KeyCode::F3 => winuser::VK_F3,
            KeyCode::F4 => winuser::VK_F4,
            KeyCode::F5 => winuser::VK_F5,
            KeyCode::F6 => winuser::VK_F6,
            KeyCode::F7 => winuser::VK_F7,
            KeyCode::F8 => winuser::VK_F8,
            KeyCode::F9 => winuser::VK_F9,
            KeyCode::F10 => winuser::VK_F10,
            KeyCode::F11 => winuser::VK_F11,
            KeyCode::F12 => winuser::VK_F12,
            KeyCode::LeftArrow => winuser::VK_LEFT,
            KeyCode::Control => winuser::VK_CONTROL,
            KeyCode::RightArrow => winuser::VK_RIGHT,
            KeyCode::DownArrow => winuser::VK_DOWN,
            KeyCode::End => winuser::VK_END,
            KeyCode::UpArrow => winuser::VK_UP,
            KeyCode::PageUp => winuser::VK_PRIOR,
            KeyCode::Alt => winuser::VK_MENU,
            KeyCode::Return => winuser::VK_RETURN,
            KeyCode::PageDown => winuser::VK_NEXT,
            KeyCode::Delete => winuser::VK_DELETE,
            KeyCode::Home => winuser::VK_HOME,
            KeyCode::Escape => winuser::VK_ESCAPE,
            KeyCode::Backspace => winuser::VK_BACK,
            KeyCode::Meta => winuser::VK_LWIN,
            KeyCode::CapsLock => winuser::VK_CAPITAL,
            KeyCode::Shift => winuser::VK_SHIFT,
            KeyCode::Tab => winuser::VK_TAB,
        };
        win_code as WinKeyCode
    }
}

#[cfg(windows)]
fn win_send_key_event(keycode: WinKeyCode, down: bool, wait: bool) {
    use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP};
    let flags = if down { 0 } else { KEYEVENTF_KEYUP };
    unsafe { keybd_event(keycode as u8, 0, flags, 0) };

    if wait {
        let ms: u64 = rand::thread_rng().gen_range(5, 20);
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

#[cfg(windows)]
fn system_toggle<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    use winapi::um::winuser::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
                              KEYEVENTF_UNICODE};
    for &flag in flags.iter() {
        win_send_key_event(WinKeyCode::from(flag), down, true);
    }
    if let Some(character) = key.character() {
        let flags = if down { 0 } else { KEYEVENTF_KEYUP };
        let mut buf = [0; 2];
        for word in character.encode_utf16(&mut buf) {
            let mut input = INPUT {
                type_: INPUT_KEYBOARD,
                u: unsafe {
                    std::mem::transmute_copy(&KEYBDINPUT {
                        wVk: 0,
                        wScan: *word,
                        dwFlags: KEYEVENTF_UNICODE | flags,
                        time: 0,
                        dwExtraInfo: 0,
                    })
                },
            };
            unsafe {
                SendInput(1, &mut input, std::mem::size_of::<INPUT>() as i32);
            }
        }
    } else {
        win_send_key_event(key.code(), down, false);
    }
}

#[cfg(target_os = "linux")]
type XKeyCode = u64;

#[cfg(target_os = "linux")]
impl From<Flag> for XKeyCode {
    fn from(flag: Flag) -> XKeyCode {
        let x_code = match flag {
            Flag::Shift => x11::keysym::XK_Shift_L,
            Flag::Control => x11::keysym::XK_Control_L,
            Flag::Alt => x11::keysym::XK_Alt_L,
            Flag::Meta => x11::keysym::XK_Meta_L,
            Flag::Help => x11::keysym::XK_Help,
        };
        x_code as XKeyCode
    }
}

#[cfg(target_os = "linux")]
impl From<KeyCode> for XKeyCode {
    fn from(code: KeyCode) -> XKeyCode {
        let x_code = match code {
            KeyCode::F1 => x11::keysym::XK_F1,
            KeyCode::F2 => x11::keysym::XK_F2,
            KeyCode::F3 => x11::keysym::XK_F3,
            KeyCode::F4 => x11::keysym::XK_F4,
            KeyCode::F5 => x11::keysym::XK_F5,
            KeyCode::F6 => x11::keysym::XK_F6,
            KeyCode::F7 => x11::keysym::XK_F7,
            KeyCode::F8 => x11::keysym::XK_F8,
            KeyCode::F9 => x11::keysym::XK_F9,
            KeyCode::F10 => x11::keysym::XK_F10,
            KeyCode::F11 => x11::keysym::XK_F11,
            KeyCode::F12 => x11::keysym::XK_F12,
            KeyCode::LeftArrow => x11::keysym::XK_leftarrow,
            KeyCode::Control => x11::keysym::XK_Control_L,
            KeyCode::RightArrow => x11::keysym::XK_rightarrow,
            KeyCode::DownArrow => x11::keysym::XK_downarrow,
            KeyCode::End => x11::keysym::XK_End,
            KeyCode::UpArrow => x11::keysym::XK_uparrow,
            KeyCode::PageUp => x11::keysym::XK_Page_Up,
            KeyCode::Alt => x11::keysym::XK_Alt_L,
            KeyCode::Return => x11::keysym::XK_Return,
            KeyCode::PageDown => x11::keysym::XK_Page_Down,
            KeyCode::Delete => x11::keysym::XK_Delete,
            KeyCode::Home => x11::keysym::XK_Home,
            KeyCode::Escape => x11::keysym::XK_Escape,
            KeyCode::Backspace => x11::keysym::XK_Delete,
            KeyCode::Meta => x11::keysym::XK_Meta_L,
            KeyCode::CapsLock => x11::keysym::XK_Caps_Lock,
            KeyCode::Shift => x11::keysym::XK_Shift_L,
            KeyCode::Tab => x11::keysym::XK_Tab,
        };
        x_code as XKeyCode
    }
}

#[cfg(target_os = "linux")]
fn x_send_key_event(display: *mut x11::xlib::Display, keycode: XKeyCode, down: bool, wait: bool) {
    unsafe {
        XTestFakeKeyEvent(
            display,
            x11::xlib::XKeysymToKeycode(display, keycode),
            down as i32,
            x11::xlib::CurrentTime,
        );
        x11::xlib::XFlush(display);
    };

    if wait {
        let ms: u64 = rand::thread_rng().gen_range(5, 20);
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

#[cfg(target_os = "linux")]
fn system_toggle<T: KeyCodeConvertible>(key: T, down: bool, flags: &[Flag]) {
    internal::X_MAIN_DISPLAY.with(|display| {
        for &flag in flags.iter() {
            x_send_key_event(*display, XKeyCode::from(flag), down, true);
        }
        x_send_key_event(*display, key.code(), down, false);
    })
}

#[cfg(target_os = "linux")]
extern "C" {
    fn XTestFakeKeyEvent(
        display: *mut x11::xlib::Display,
        keycode: u8,
        is_press: i32,
        delay: x11::xlib::Time,
    ) -> i32;
}
