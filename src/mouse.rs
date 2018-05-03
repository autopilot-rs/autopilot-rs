//! This module contains functions for getting the current state of and
//! controlling the mouse cursor.
//!
//! Unless otherwise stated, coordinates are those of a screen coordinate
//! system, where the origin is at the top left.

use geometry::Point;
use screen;
use std;

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton,
                           ScrollEventUnit};
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSourceStateID::HIDSystemState;
#[cfg(target_os = "macos")]
use core_graphics::geometry::CGPoint;
#[cfg(windows)]
use winapi::shared::minwindef::DWORD;

#[cfg(target_os = "linux")]
use internal;
#[cfg(target_os = "linux")]
use x11;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Button {
    Left,
    Middle,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Debug)]
pub enum MouseError {
    OutOfBounds,
}

/// Gradually moves the mouse to a coordinate in a straight line in the given
/// time frame (in seconds). If no duration is given a 1 millisecond delay is
/// defaulted to between mouse movements.
///
/// Returns `MouseError` if coordinate is outside the screen boundaries.
pub fn smooth_move(destination: Point, duration: Option<f64>) -> Result<(), MouseError> {
    if !screen::is_point_visible(destination) {
        return Err(MouseError::OutOfBounds);
    }

    let start_position = location();
    let distance = (start_position.x - destination.x).hypot(start_position.y - destination.y);
    let step_count = distance.ceil() as i64;
    let interval: u64 = duration
        .map(|d| (d * 1000.0) / distance)
        .unwrap_or(1.0)
        .round() as u64;

    for step in 1..step_count + 1 {
        let position = Point::new(
            (destination.x - start_position.x) * (step as f64 / step_count as f64)
                + start_position.x,
            (destination.y - start_position.y) * (step as f64 / step_count as f64)
                + start_position.y,
        );

        try!(move_to(position));
        std::thread::sleep(std::time::Duration::from_millis(interval));
    }

    Ok(())
}

/// A convenience wrapper around `toggle()` that holds down and then releases
/// the given mouse button. Delay between pressing and releasing the key can be
/// controlled using the `delay_ms` parameter. If `delay` is not given, the
/// value defaults to 100 ms.
pub fn click(button: Button, delay_ms: Option<u64>) {
    toggle(button, true);
    std::thread::sleep(std::time::Duration::from_millis(delay_ms.unwrap_or(100)));
    toggle(button, true);
}

/// Immediately moves the mouse to the given coordinate.
///
/// Returns `MouseError` if coordinate is outside the screen boundaries.
pub fn move_to(point: Point) -> Result<(), MouseError> {
    if !screen::is_point_visible(point) {
        Err(MouseError::OutOfBounds)
    } else {
        system_move_to(point);
        Ok(())
    }
}

/// Returns the current position of the mouse cursor.
pub fn location() -> Point {
    system_location()
}

/// Holds down or releases a mouse button in the current position.
pub fn toggle(button: Button, down: bool) {
    system_toggle(button, down);
}

/// Performs a scroll event in a direction a given number of times.
pub fn scroll(direction: ScrollDirection, clicks: u32) {
    system_scroll(direction, clicks);
}

#[cfg(target_os = "macos")]
impl Button {
    fn event_type(&self, down: bool) -> CGEventType {
        use core_graphics::event::CGEventType::*;
        match (*self, down) {
            (Button::Left, true) => LeftMouseDown,
            (Button::Left, false) => LeftMouseUp,
            (Button::Right, true) => RightMouseDown,
            (Button::Right, false) => RightMouseUp,
            (Button::Middle, true) => OtherMouseDown,
            (Button::Middle, false) => OtherMouseUp,
        }
    }
}

#[cfg(target_os = "macos")]
impl From<Button> for CGMouseButton {
    fn from(button: Button) -> CGMouseButton {
        use core_graphics::event::CGMouseButton::*;
        match button {
            Button::Left => Left,
            Button::Middle => Center,
            Button::Right => Right,
        }
    }
}

#[cfg(target_os = "macos")]
fn system_move_to(point: Point) {
    let point = CGPoint::from(point);
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event =
        CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left);
    event.unwrap().post(CGEventTapLocation::HID);
}

#[cfg(target_os = "macos")]
fn system_location() -> Point {
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event = CGEvent::new(source).unwrap();
    Point::from(event.location())
}

#[cfg(target_os = "macos")]
fn system_toggle(button: Button, down: bool) {
    let point = CGPoint::from(location());
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event_type = button.event_type(down);
    let event = CGEvent::new_mouse_event(source, event_type, point, CGMouseButton::from(button));
    event.unwrap().post(CGEventTapLocation::HID);
}

#[cfg(target_os = "macos")]
fn system_scroll(direction: ScrollDirection, clicks: u32) {
    for _ in 0..clicks {
        let wheel_count = if direction == ScrollDirection::Up {
            10
        } else {
            -10
        };
        let source = CGEventSource::new(HIDSystemState).unwrap();
        let event = CGEvent::new_scroll_event(source, ScrollEventUnit::LINE, 1, wheel_count, 0, 0);
        event.unwrap().post(CGEventTapLocation::HID);
    }
}

#[cfg(windows)]
fn mouse_event_for_button(button: Button, down: bool) -> DWORD {
    use winapi::um::winuser::{MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
                              MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP};
    match (button, down) {
        (Button::Left, true) => MOUSEEVENTF_LEFTDOWN,
        (Button::Left, false) => MOUSEEVENTF_LEFTUP,
        (Button::Right, true) => MOUSEEVENTF_RIGHTDOWN,
        (Button::Right, false) => MOUSEEVENTF_RIGHTUP,
        (Button::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
        (Button::Middle, false) => MOUSEEVENTF_MIDDLEUP,
    }
}

#[cfg(windows)]
fn system_move_to(point: Point) {
    use winapi::um::winuser::{mouse_event, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE};
    let screen_size = screen::size().scaled(screen::scale());
    let scaled_point = point.scaled(screen::scale());
    let x = scaled_point.x as DWORD * 0xFFFF / screen_size.width as DWORD;
    let y = scaled_point.y as DWORD * 0xFFFF / screen_size.height as DWORD;
    unsafe {
        mouse_event(MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE, x, y, 0, 0);
    };
}

#[cfg(windows)]
fn system_location() -> Point {
    use winapi::shared::windef::POINT;
    use winapi::um::winuser::GetCursorPos;
    let mut point: POINT = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut point);
    }
    Point::from(point).scaled(screen::scale())
}

#[cfg(windows)]
fn system_toggle(button: Button, down: bool) {
    use winapi::um::winuser::mouse_event;
    unsafe {
        mouse_event(mouse_event_for_button(button, down), 0, 0, 0, 0);
    };
}

#[cfg(windows)]
fn system_scroll(direction: ScrollDirection, clicks: u32) {
    use winapi::um::winuser::{mouse_event, MOUSEEVENTF_WHEEL, WHEEL_DELTA};
    unsafe {
        let multiplier: DWORD = if direction == ScrollDirection::Up {
            1
        } else {
            -1
        };
        mouse_event(
            MOUSEEVENTF_WHEEL,
            0,
            0,
            WHEEL_DELTA as DWORD * clicks as DWORD * multiplier,
            0,
        );
    };
}

#[cfg(target_os = "linux")]
impl From<Button> for XButton {
    fn from(button: Button) -> XButton {
        match button {
            Button::Left => X_BUTTON_LEFT,
            Button::Middle => X_BUTTON_MIDDLE,
            Button::Right => X_BUTTON_RIGHT,
        }
    }
}

#[cfg(target_os = "linux")]
impl From<ScrollDirection> for XButton {
    fn from(direction: ScrollDirection) -> XButton {
        match direction {
            ScrollDirection::Up => X_BUTTON_SCROLL_UP,
            ScrollDirection::Down => X_BUTTON_SCROLL_DOWN,
        }
    }
}

#[cfg(target_os = "linux")]
fn system_move_to(point: Point) {
    use scopeguard::guard;
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
        let scaled_point = point.scaled(screen::scale());
        let root_window = guard(x11::xlib::XDefaultRootWindow(*display), |w| {
            x11::xlib::XDestroyWindow(*display, *w);
        });
        x11::xlib::XWarpPointer(
            *display,
            0,
            *root_window,
            0,
            0,
            0,
            0,
            scaled_point.x as i32,
            scaled_point.y as i32,
        );
        x11::xlib::XFlush(*display);
    });
}

#[cfg(target_os = "linux")]
fn system_location() -> Point {
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
        let root_window = x11::xlib::XDefaultRootWindow(*display);
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut unused_a: x11::xlib::Window = 0;
        let mut unused_b: x11::xlib::Window = 0;
        let mut unused_c: i32 = 0;
        let mut unused_d: i32 = 0;
        let mut unused_e: u32 = 0;
        x11::xlib::XQueryPointer(
            *display,
            root_window,
            &mut unused_a,
            &mut unused_b,
            &mut x,
            &mut y,
            &mut unused_c,
            &mut unused_d,
            &mut unused_e,
        );
        Point::new(x as f64, y as f64).scaled(screen::scale())
    })
}

#[cfg(target_os = "linux")]
fn send_button_event(display: *mut x11::xlib::Display, button: XButton, down: bool) {
    unsafe {
        XTestFakeButtonEvent(
            display,
            XButton::from(button),
            down as i32,
            x11::xlib::CurrentTime,
        );
        x11::xlib::XFlush(display);
    };
}

#[cfg(target_os = "linux")]
fn system_toggle(button: Button, down: bool) {
    internal::X_MAIN_DISPLAY.with(|display| {
        send_button_event(*display, XButton::from(button), down);
    });
}

#[cfg(target_os = "linux")]
fn system_scroll(direction: ScrollDirection, clicks: u32) {
    internal::X_MAIN_DISPLAY.with(|display| {
        for _ in 0..clicks {
            send_button_event(*display, XButton::from(direction), true);
            send_button_event(*display, XButton::from(direction), false);
        }
    });
}

#[cfg(target_os = "linux")]
type XButton = u32;

#[cfg(target_os = "linux")]
const X_BUTTON_LEFT: XButton = 1;
#[cfg(target_os = "linux")]
const X_BUTTON_MIDDLE: XButton = 2;
#[cfg(target_os = "linux")]
const X_BUTTON_RIGHT: XButton = 3;
#[cfg(target_os = "linux")]
const X_BUTTON_SCROLL_UP: XButton = 4;
#[cfg(target_os = "linux")]
const X_BUTTON_SCROLL_DOWN: XButton = 5;

#[cfg(target_os = "linux")]
extern "C" {
    fn XTestFakeButtonEvent(
        display: *mut x11::xlib::Display,
        button: u32,
        is_press: i32,
        delay: x11::xlib::Time,
    ) -> i32;
}
