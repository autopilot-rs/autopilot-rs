//! This module contains functions for getting the current state of and
//! controlling the mouse cursor.
//!
//! Unless otherwise stated, coordinates are those of a screen coordinate
//! system, where the origin is at the top left.
extern crate rand;

use geometry::Point;
use screen;
use self::rand::Rng;
use std::{thread, time};

#[cfg(target_os = "macos")]
use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSource;
#[cfg(target_os = "macos")]
use core_graphics::event_source::CGEventSourceStateID::HIDSystemState;
#[cfg(target_os = "macos")]
use core_graphics::geometry::CGPoint;

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
        thread::sleep(time::Duration::from_millis(duration));
    }

    Ok(())
}

/// A convenience wrapper around `toggle()` that holds down and then releases
/// the given mouse button.
pub fn click(button: Button) {
    let ms: u64 = rand::thread_rng().gen_range(50, 100);
    toggle(button, true);
    thread::sleep(time::Duration::from_millis(ms));
    toggle(button, true);
}

/// Immediately moves the mouse to the given coordinate.
///
/// Returns `MouseError` if coordinate is outside the screen boundaries.
pub fn move_to(point: Point) -> Result<(), MouseError> {
    if !screen::is_point_visible(point) {
        Err(MouseError::OutOfBounds)
    } else {
        if cfg!(target_os = "macos") {
            macos_move_to(point);
            Ok(())
        } else {
            panic!("Unsupported OS");
        }
    }
}

/// Returns the current position of the mouse cursor.
pub fn location() -> Point {
    if cfg!(target_os = "macos") {
        macos_location()
    } else {
        panic!("Unsupported OS");
    }
}

/// Holds down or releases a mouse button in the current position.
pub fn toggle(button: Button, down: bool) {
    if cfg!(target_os = "macos") {
        macos_toggle(button, down);
    } else {
        panic!("Unsupported OS");
    }
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
fn macos_move_to(point: Point) {
    let point = CGPoint::from(point);
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event =
        CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left);
    event.unwrap().post(CGEventTapLocation::HID);
}

#[cfg(target_os = "macos")]
fn macos_location() -> Point {
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event = CGEvent::new(source).unwrap();
    Point::from(event.location())
}

#[cfg(target_os = "macos")]
fn macos_toggle(button: Button, down: bool) {
    let point = CGPoint::from(location());
    let source = CGEventSource::new(HIDSystemState).unwrap();
    let event_type = button.event_type(down);
    let event = CGEvent::new_mouse_event(source, event_type, point, CGMouseButton::from(button));
    event.unwrap().post(CGEventTapLocation::HID);
}
