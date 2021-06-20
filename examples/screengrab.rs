extern crate autopilot;
extern crate image;
use autopilot::geometry::{Point, Rect, Size};
use std::path::Path;

fn main() {
    let bmp = autopilot::bitmap::capture_screen().expect("Unable to capture screen");
    let portion = autopilot::bitmap::capture_screen_portion(Rect::new(
        Point::new(100.0, 100.0),
        Size::new(100.0, 100.0),
    )).expect("Unable to capture screen portion");
    let bmp_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("screenshot.png");
    let portion_path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("screenshot_cropped.png");
    let _ = bmp.image
        .save(&bmp_path)
        .expect("Unable to save screenshot");
    let _ = portion
        .image
        .save(&portion_path)
        .expect("Unable to save cropped screenshot");
    println!("Scale factor {}", autopilot::screen::scale());
    println!("Screen size {}", autopilot::screen::size());
    println!("Saved screenshot at {}", bmp_path.to_str().unwrap_or(""));
    println!(
        "Saved cropped screenshot at {}",
        portion_path.to_str().unwrap_or("")
    );
}
