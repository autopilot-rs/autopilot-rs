use geometry::{Point, Rect, Size};

#[cfg(target_os = "macos")]
use core_graphics::base::CGFloat;
#[cfg(target_os = "macos")]
use core_graphics::geometry::{CGPoint, CGRect, CGSize};

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
