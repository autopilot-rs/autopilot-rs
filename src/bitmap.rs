//! This module defines the struct `Bitmap` for accessing bitmaps and
//! searching for bitmaps on-screen.
//!
//! It also defines functions for taking screenshots of the screen.
extern crate image;

use geometry::{Point, Rect, Size};
use screen;
use image::{DynamicImage, GenericImage, ImageError, ImageFormat, ImageResult, Pixel, Rgba,
            RgbaImage};
use libc::size_t;
use libc;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSImage, NSPasteboard};
#[cfg(target_os = "macos")]
use cocoa::base::nil;
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSArray, NSData};
#[cfg(target_os = "macos")]
use core_graphics::base::CGFloat;
#[cfg(target_os = "macos")]
use core_graphics::context::CGContext;
#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;
#[cfg(target_os = "macos")]
use core_graphics::geometry::{CGRect, CGSize, CG_ZERO_POINT};
#[cfg(target_os = "macos")]
use core_graphics::image::{CGImage, CGImageAlphaInfo, CGImageByteOrderInfo};

pub struct Bitmap {
    pub image: DynamicImage,
    pub size: Size,
    pub scale: f64,
}

impl Bitmap {
    #[inline]
    /// Creates a bitmap from the given `DynamicImage`, and scale if given
    /// (defaults to 1).
    pub fn new(image: DynamicImage, scale: Option<f64>) -> Bitmap {
        let scale: f64 = scale.unwrap_or(1.0);
        Bitmap {
            size: Size::new(image.width() as f64, image.height() as f64),
            image: image,
            scale: scale,
        }
    }

    #[inline]
    /// Returns bounds of bitmap as a rect, with an origin of zero.
    pub fn bounds(&self) -> Rect {
        Rect::new(Point::ZERO, self.size)
    }

    /// Copies image to pasteboard. Currently only supported on Windows and
    /// macOS.
    pub fn copy_to_pasteboard(&self) -> ImageResult<()> {
        if cfg!(target_os = "macos") {
            self.macos_copy_to_pasteboard()
        } else {
            panic!("Unsupported OS");
        }
    }

    /// Returns new Bitmap created from a portion of another.
    pub fn cropped(&mut self, rect: Rect) -> ImageResult<Bitmap> {
        if !self.bounds().is_rect_visible(rect) {
            Err(ImageError::DimensionError)
        } else {
            let rect = rect.scaled(self.multiplier()).round();
            let cropped_image = self.image.crop(
                rect.origin.x as u32,
                rect.origin.y as u32,
                rect.size.width as u32,
                rect.size.height as u32,
            );
            Ok(Bitmap::new(cropped_image, Some(self.scale)))
        }
    }

    pub fn get_pixel(&self, point: Point) -> Rgba<u8> {
        let point = point.scaled(self.multiplier()).round();
        self.image.get_pixel(point.x as u32, point.y as u32)
    }

    /// Attempts to find `color` inside `rect` in `bmp` from the given
    /// `start_point`. Returns coordinates if found, or `None` if not. If
    /// `rect` is `None`, `bmp.bounds()` is used instead. If `start_point` is
    /// `None`, the origin of `rect` is used.
    ///
    /// Tolerance is defined as a float in the range from 0 to 1, where 0 is
    /// an exact match and 1 matches anything.
    pub fn find_color(
        &self,
        needle: Rgba<u8>,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> Option<Point> {
        let tolerance = tolerance.unwrap_or(0.0);
        self.find(rect, start_point, |point| {
            colors_match(needle, self.get_pixel(point), tolerance)
        })
    }

    /// Returns list of all coordinates inside `rect` in `bmp` matching
    /// `color` from the given `start_point`. If `rect` is `None`,
    /// `bmp.bounds()` is used instead. If `start_point` is `None`, the origin
    /// of `rect` is used.
    pub fn find_every_color(
        &self,
        needle: Rgba<u8>,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> Vec<Point> {
        let mut points: Vec<Point> = Vec::new();
        {
            let tolerance = tolerance.unwrap_or(0.0);
            let mut matched = |point| {
                points.push(point);
            };
            self.find_all(
                rect,
                start_point,
                &(|point| colors_match(needle, self.get_pixel(point), tolerance)),
                &mut matched,
            );
        }
        points
    }

    /// Returns count of color in bitmap. Functionally equivalent to:
    ///
    ///     find_every_color(color, tolerance, rect, start_point).count()
    ///
    pub fn count_of_color(
        &self,
        needle: Rgba<u8>,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> u64 {
        let mut count: u64 = 0;
        {
            let tolerance = tolerance.unwrap_or(0.0);
            let mut matched = |_| {
                count += 1;
            };
            self.find_all(
                rect,
                start_point,
                &(|point| colors_match(needle, self.get_pixel(point), tolerance)),
                &mut matched,
            );
        }
        count
    }

    /// Attempts to find `needle` inside `rect` in `bmp` from the given
    /// `start_point`. Returns coordinates if found, or `None` if not. If
    /// `rect` is `None`, `bmp.bounds()` is used instead. If `start_point` is
    /// `None`, the origin of `rect` is used.
    ///
    /// Tolerance is defined as a float in the range from 0 to 1, where 0 is
    /// an exact match and 1 matches anything.
    pub fn find_bitmap(
        &self,
        needle: &Bitmap,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> Option<Point> {
        if self.is_needle_oversized(needle) {
            return None;
        }

        self.find(rect, start_point, |pt| {
            self.is_needle_at(pt, needle, tolerance)
        })
    }

    /// Returns list of all coordinates inside `rect` in `bmp` matching
    /// `needle` from the given `start_point`. If `rect` is `None`,
    /// `bmp.bounds` is used instead. If `start_point` is `None`, the origin
    /// of `rect` is used.
    pub fn find_every_bitmap(
        &self,
        needle: &Bitmap,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> Vec<Point> {
        if self.is_needle_oversized(needle) {
            return Vec::new();
        }

        let mut points: Vec<Point> = Vec::new();
        {
            let mut matched = |point| {
                points.push(point);
            };
            self.find_all(
                rect,
                start_point,
                &(|pt| self.is_needle_at(pt, needle, tolerance)),
                &mut matched,
            );
        }
        points
    }

    /// Returns count of occurrences of `needle` in `bmp`. Functionally equivalent to:
    ///
    ///     find_every_bitmap(color, tolerance, rect, start_point).count()
    ///
    pub fn count_of_bitmap(
        &self,
        needle: &Bitmap,
        tolerance: Option<f64>,
        rect: Option<Rect>,
        start_point: Option<Point>,
    ) -> u64 {
        if self.is_needle_oversized(needle) {
            return 0;
        }

        let mut count: u64 = 0;
        {
            let mut matched = |_| {
                count += 1;
            };
            self.find_all(
                rect,
                start_point,
                &(|pt| self.is_needle_at(pt, needle, tolerance)),
                &mut matched,
            );
        }
        count
    }

    #[inline]
    fn multiplier(&self) -> f64 {
        1.0 / self.scale
    }

    #[inline]
    fn is_needle_oversized(&self, needle: &Bitmap) -> bool {
        needle.bounds().size.width > self.bounds().size.width
            && needle.bounds().size.height > self.bounds().size.height
    }

    fn is_needle_at(&self, pt: Point, needle: &Bitmap, tolerance: Option<f64>) -> bool {
        let bounds = needle.bounds();
        for x in bounds.origin.x as u64..bounds.max_x() as u64 {
            for y in bounds.origin.y as u64..bounds.max_y() as u64 {
                let needle_point = Point::new(x as f64, y as f64);
                let haystack_point = Point::new(pt.x + needle_point.x, pt.y + needle_point.y);
                if !self.bounds().is_point_visible(haystack_point) {
                    return false;
                }

                let c1 = needle.get_pixel(needle_point);
                let c2 = self.get_pixel(haystack_point);
                if !colors_match(c1, c2, tolerance.unwrap_or(0.0f64)) {
                    return false;
                }
            }
        }

        true
    }

    fn find<F: Fn(Point) -> bool>(
        &self,
        rect: Option<Rect>,
        start_point: Option<Point>,
        predicate: F,
    ) -> Option<Point> {
        let rect = rect.unwrap_or(self.bounds());
        let start_point = start_point.unwrap_or(self.bounds().origin);
        if !self.bounds().is_rect_visible(rect) {
            panic!(
                "invalid rect: {} outside of image bounds ({})",
                rect,
                self.bounds()
            );
        }
        if !self.bounds().is_point_visible(start_point) {
            panic!(
                "invalid start point: {} outside of image bounds ({})",
                start_point,
                self.bounds()
            );
        }

        // TODO: Switch the Boyer-Moore algorithm for image search or use this instead
        // http://bit.ly/1EIEIfr.
        let start_point = start_point.scaled(self.multiplier()).round();
        let rect = rect.scaled(self.multiplier()).round();
        for x in start_point.x as u64..rect.max_x() as u64 {
            for y in start_point.y as u64..rect.max_y() as u64 {
                let point = Point::new(x as f64, y as f64);
                if predicate(point) {
                    return Some(point);
                }
            }
        }

        None
    }

    fn find_all<'a>(
        &self,
        rect: Option<Rect>,
        start_point: Option<Point>,
        predicate: &'a Fn(Point) -> bool,
        matched: &'a mut FnMut(Point) -> (),
    ) {
        let rect = rect.unwrap_or(self.bounds());
        let mut start_point = start_point.unwrap_or(self.bounds().origin);
        loop {
            if let Some(point) = self.find(Some(rect), Some(start_point), predicate) {
                matched(point);
                if let Some(next_point) = rect.iter_point(point) {
                    start_point = next_point;
                    continue;
                }
            }

            break;
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_copy_to_pasteboard(&self) -> ImageResult<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let result = self.image.save(&mut buffer, ImageFormat::PNG);
        match result {
            Ok(_) => unsafe {
                let data = NSData::dataWithBytesNoCopy_length_(
                    nil,
                    buffer.as_ptr() as *const libc::c_void,
                    buffer.len() as u64,
                );
                let image = NSImage::initWithData_(NSImage::alloc(nil), data);
                let objects = NSArray::arrayWithObject(nil, image);
                let pasteboard = NSPasteboard::generalPasteboard(nil);
                pasteboard.clearContents();
                pasteboard.writeObjects(objects);
                result
            },
            _ => result,
        }
    }
}

/// Returns true if the given two colors are sufficiently similar.
///
/// Tolerance is defined as a double in the range from 0 to 1, where 0 is an
/// exact match and 1 matches anything.
fn colors_match(c1: Rgba<u8>, c2: Rgba<u8>, tolerance: f64) -> bool {
    if tolerance <= 0.0 {
        return c1 == c2;
    }

    let (r1, g1, b1, _) = c1.channels4();
    let (r2, g2, b2, _) = c2.channels4();
    let d1: f64 = (r1 as f64 - r2 as f64).abs();
    let d2: f64 = (g1 as f64 - g2 as f64).abs();
    let d3: f64 = (b1 as f64 - b2 as f64).abs();
    (d1 * d1 + d2 * d2 + d3 * d3).sqrt() <= tolerance * MAX_TOLERANCE_DELTA
}

const MAX_TOLERANCE_DELTA: f64 = 441.6729559301; // => (3.0f64 * 255.0f64 * 255.0f64).sqrt();

/// Returns a screengrab of the entire main display.
pub fn capture_screen() -> ImageResult<Bitmap> {
    capture_screen_portion(Rect::new(Point::ZERO, screen::size()))
}

/// Returns a screengrab of the given portion of the main display.
pub fn capture_screen_portion(rect: Rect) -> ImageResult<Bitmap> {
    if !screen::is_rect_visible(rect) {
        Err(ImageError::DimensionError)
    } else if cfg!(target_os = "macos") {
        if let Some(image) = CGDisplay::screenshot(CGRect::from(rect), 0, 0, 0) {
            macos_load_cgimage(image)
        } else {
            Err(ImageError::NotEnoughData)
        }
    } else {
        panic!("Unsupported OS");
    }
}

#[cfg(target_os = "macos")]
fn macos_load_cgimage(image: CGImage) -> ImageResult<Bitmap> {
    let width: size_t = image.width();
    let height: size_t = image.height();
    let bits_per_component: size_t = image.bits_per_component();
    let bytes_per_row: size_t = image.bytes_per_row();
    let space = image.color_space();
    let flags: u32 = CGImageByteOrderInfo::CGImageByteOrder32Big as u32
        | CGImageAlphaInfo::CGImageAlphaNoneSkipLast as u32;
    let mut context = CGContext::create_bitmap_context(
        None,
        width,
        height,
        bits_per_component,
        bytes_per_row,
        &space,
        flags,
    );
    let rect = CGRect {
        origin: CG_ZERO_POINT,
        size: CGSize::new(width as CGFloat, height as CGFloat),
    };

    context.draw_image(rect, &image);

    let buffer: &[u8] = context.data();
    let image = RgbaImage::from_raw(width as u32, height as u32, buffer.to_vec()).unwrap();
    // let dynimage = DynamicImage::ImageRgb8(DynamicImage::ImageRgba8(image).to_rgb());
    let dynimage = DynamicImage::ImageRgba8(image);
    let bmp = Bitmap::new(dynimage, Some(screen::scale()));
    // let mut result = DynamicImage::new_rgb8(width as u32, height as u32);
    // for x in 0..width {
    //    for y in 0..height {
    //        let offset = bytes_per_row * y + bytes_per_pixel * x;
    //        let (r, g, b) = (buffer[offset], buffer[offset + 1], buffer[offset + 2]);
    //        result.put_pixel(x as u32, y as u32, Rgba([r, g, b, 255]));
    //    }
    // }

    Ok(bmp)
}
