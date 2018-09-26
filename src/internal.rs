#[cfg(windows)]
use geometry::Point;
#[cfg(target_os = "macos")]
use geometry::{Point, Rect, Size};

#[cfg(target_os = "macos")]
use core_graphics::base::CGFloat;
#[cfg(target_os = "macos")]
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
#[cfg(target_os = "linux")]
use std;
#[cfg(windows)]
use winapi::shared::windef::POINT;
#[cfg(target_os = "linux")]
use x11;

#[cfg(target_os = "macos")]
impl From<Point> for CGPoint {
    fn from(point: Point) -> CGPoint {
        CGPoint::new(point.x as CGFloat, point.y as CGFloat)
    }
}

#[cfg(target_os = "macos")]
impl From<CGPoint> for Point {
    fn from(point: CGPoint) -> Point {
        Point::new(point.x as f64, point.y as f64)
    }
}

#[cfg(windows)]
impl From<POINT> for Point {
    fn from(point: POINT) -> Point {
        Point::new(point.x as f64, point.y as f64)
    }
}

#[cfg(target_os = "macos")]
impl From<Size> for CGSize {
    fn from(size: Size) -> CGSize {
        CGSize::new(size.width as CGFloat, size.height as CGFloat)
    }
}

#[cfg(target_os = "macos")]
impl From<CGSize> for Size {
    fn from(size: CGSize) -> Size {
        Size::new(size.width as f64, size.height as f64)
    }
}

#[cfg(target_os = "macos")]
impl From<Rect> for CGRect {
    fn from(rect: Rect) -> CGRect {
        CGRect {
            origin: CGPoint::from(rect.origin),
            size: CGSize::from(rect.size),
        }
    }
}

#[cfg(target_os = "linux")]
thread_local!(pub static X_MAIN_DISPLAY: *mut x11::xlib::Display = unsafe {
    x11::xlib::XOpenDisplay(std::ptr::null())
});

#[cfg(target_os = "linux")]
thread_local!(pub static X_SCALE_FACTOR: f64 = {
    use std::ffi::{CString, CStr};
    use libc;
    // From https://github.com/glfw/glfw/issues/1019#issuecomment-302772498
    X_MAIN_DISPLAY.with(|display| unsafe {
        let screen = x11::xlib::XDefaultScreen(*display);
        let width = x11::xlib::XDisplayWidth(*display, screen) as f64;
        let width_mm = x11::xlib::XDisplayWidthMM(*display, screen) as f64;

        // Default to display-wide DPI if Xft.dpi is unset.
        let mut dpi = width * 25.4 / width_mm;

        // Prefer value set in xrdb.
        let rms = x11::xlib::XResourceManagerString(*display);
        if rms != std::ptr::null_mut() {
            let db = x11::xlib::XrmGetStringDatabase(rms);
            if db != std::ptr::null_mut() {
                defer!({
                    x11::xlib::XrmDestroyDatabase(db);
                });
                let mut value = x11::xlib::XrmValue{
                    size: 0,
                    addr: std::ptr::null_mut(),
                };

                let mut value_type: *mut libc::c_char = std::ptr::null_mut();
                if x11::xlib::XrmGetResource(
                    db,
                    CString::new("Xft.dpi").unwrap().as_ptr(),
                    CString::new("String").unwrap().as_ptr(),
                    &mut value_type,
                    &mut value
                ) != 0 && value.addr != std::ptr::null_mut() {
                    let value_addr: &CStr = CStr::from_ptr(value.addr);
                    if let Some(parsed_dpi) = value_addr
                        .to_str()
                        .ok()
                        .and_then(|s| s.parse::<f64>().ok()) {
                        dpi = parsed_dpi;
                    }
                }
            }
        }
        let scale = dpi / 96.0;
        (scale * 100.0).floor() / 100.0
    })
});
