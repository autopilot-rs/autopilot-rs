//! This module contains functions for working with the screen.
extern crate image;
use bitmap;
use geometry::{Point, Rect, Size};
use self::image::{GenericImage, ImageResult, Rgba};

#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;
#[cfg(target_os = "linux")]
use internal;
#[cfg(target_os = "linux")]
use x11;

/// Returns the size of the main screen in points.
pub fn size() -> Size {
    system_size()
}

/// Returns the scale of the main screen, i.e. how many pixels are in a point.
pub fn scale() -> f64 {
    system_scale()
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
fn system_size() -> Size {
    Size::from(CGDisplay::main().bounds().size)
}

#[cfg(target_os = "macos")]
fn system_scale() -> f64 {
    let mode = CGDisplay::main().display_mode().unwrap();
    mode.pixel_height() as f64 / mode.height() as f64
}

#[cfg(windows)]
fn system_size() -> Size {
    use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    let scale_factor = scale();
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as f64;
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as f64;
    Size::new(width, height).scaled(1.0 / scale_factor)
}

#[cfg(windows)]
fn system_scale() -> f64 {
    use winapi::um::winuser::GetDesktopWindow;
    unsafe { SetProcessDPIAware() };
    let window = unsafe { GetDesktopWindow() };
    let dpi = unsafe { GetDpiForWindow(window) };
    dpi as f64 / 96.0
}

#[cfg(target_os = "linux")]
fn system_size() -> Size {
    internal::X_MAIN_DISPLAY.with(|display| unsafe {
        let screen = x11::xlib::XDefaultScreen(*display);
        let width = x11::xlib::XDisplayWidth(*display, screen) as f64;
        let height = x11::xlib::XDisplayHeight(*display, screen) as f64;
        Size::new(width, height)
    })
}

#[cfg(target_os = "linux")]
fn system_scale() -> f64 {
    1.0
}

#[cfg(windows)]
use winapi::shared::windef::HWND;
#[cfg(windows)]
use libc;

#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    fn SetProcessDPIAware();
    fn GetDpiForWindow(hWnd: HWND) -> libc::c_uint;
}
