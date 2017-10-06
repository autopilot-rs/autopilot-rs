//! This module contains functions for working with the screen.
extern crate image;
use bitmap;
use geometry::{Point, Rect, Size};
use self::image::{GenericImage, ImageResult, Rgba};

#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;

/// Returns the size of the main screen.
pub fn size() -> Size {
    if cfg!(target_os = "macos") {
        macos_size()
    } else {
        panic!("Unsupported OS");
    }
}

/// Returns the scale of the main screen.
pub fn scale() -> f64 {
    if cfg!(target_os = "macos") {
        macos_scale()
    } else {
        panic!("Unsupported OS");
    }
}

/// Returns whether the given point is inside the main screen boundaries.
pub fn is_point_visible(point: Point) -> bool {
    Rect::new(Point::ZERO, size()).is_point_visible(point)
}

/// Returns whether the given rect is inside the main screen boundaries.
pub fn is_rect_visible(rect: Rect) -> bool {
    Rect::new(Point::ZERO, size()).is_rect_visible(rect)
}

/// A convenience method that returns the RGB color at the given point on the
/// main display.
pub fn get_color(point: Point) -> ImageResult<Rgba<u8>> {
    let bmp = try!(bitmap::capture_screen_portion(Rect::new(
        point,
        Size::new(1.0, 1.0)
    )));
    Ok(bmp.image.get_pixel(0, 0))
}

#[cfg(target_os = "macos")]
fn macos_size() -> Size {
    Size::from(CGDisplay::main().bounds().size)
}

#[cfg(target_os = "macos")]
fn macos_scale() -> f64 {
    let mode = CGDisplay::main().display_mode().unwrap();
    mode.pixel_height() as f64 / mode.height() as f64
}
