//! This module contains functions for getting the current state of and
//! controlling the mouse cursor.
//!
//! Unless otherwise stated, coordinates are those of a screen coordinate
//! system, where the origin is at the top left.

use geometry::Point;
use screen;
use rand;
use rand::Rng;
use std;

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSourceStateID::HIDSystemState;
#[cfg(target_os = "macos")]
use core_graphics::geometry::CGPoint;

#[cfg(target_os = "linux")]
use x11;
#[cfg(target_os = "linux")]
use internal;

#[derive(Copy, Clone, Debug)]
pub enum Button {
    Left,
    Middle,
    Right,
}

#[derive(Debug)]
pub enum MouseError {
    OutOfBounds,
}

/// Gradually moves the mouse to the given coordinate in a straight line.
///
/// Returns `MouseError` if coordinate is outside the screen boundaries.
pub fn smooth_move(destination: Point) -> Result<(), MouseError> {
    let mut position = location();
    let mut velocity = Point::ZERO;
    let screen_size = screen::size();

    loop {
        let distance = (position.x - destination.x).hypot(position.y - destination.y);
        if distance <= 1.0 {
            break;
        }

        let gravity: f64 = rand::thread_rng().gen_range(5.0, 500.0);
        velocity.x += (gravity * (destination.x - position.x)) / distance;
        velocity.y += (gravity * (destination.y - position.y)) / distance;

        // Normalize velocity to get a unit vector of length 1.
        let velo_distance: f64 = velocity.x.hypot(velocity.y);
        velocity.x /= velo_distance;
        velocity.y /= velo_distance;

        position.x += (velocity.x + 0.5).floor();
        position.y += (velocity.y + 0.5).floor();

        // Make sure we are still in the screen boundaries.
        if position.x < 0.0 || position.y < 0.0 || position.x >= screen_size.width
            || position.y >= screen_size.height
        {
            return Err(MouseError::OutOfBounds);
        }

        try!(move_to(position));
        let duration: u64 = rand::thread_rng().gen_range(1, 3);
        std::thread::sleep(std::time::Duration::from_millis(duration));
    }

    Ok(())
}

/// A convenience wrapper around `toggle()` that holds down and then releases
/// the given mouse button.
pub fn click(button: Button) {
    let ms: u64 = rand::thread_rng().gen_range(50, 100);
    toggle(button, true);
    std::thread::sleep(std::time::Duration::from_millis(ms));
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
fn system_move_to(point: Point) {
    use scopeguard::guard;
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
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
            point.x as i32,
            point.y as i32,
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
        Point::new(x as f64, y as f64)
    })
}

#[cfg(target_os = "linux")]
fn system_toggle(button: Button, down: bool) {
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
        XTestFakeButtonEvent(
            *display,
            XButton::from(button),
            down as i32,
            x11::xlib::CurrentTime,
        );
        x11::xlib::XFlush(*display);
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
extern "C" {
    fn XTestFakeButtonEvent(
        display: *mut x11::xlib::Display,
        button: u32,
        is_press: i32,
        delay: x11::xlib::Time,
    ) -> i32;
}
