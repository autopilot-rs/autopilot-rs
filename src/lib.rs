//! autopilot is a simple, cross-platform GUI automation library for Rust.
mod internal;
pub mod alert;
pub mod bitmap;
pub mod geometry;
pub mod key;
pub mod mouse;
pub mod screen;

extern crate image;
extern crate libc;
extern crate rand;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(target_os = "macos")]
extern crate cocoa;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate core_graphics;
