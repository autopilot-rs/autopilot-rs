extern crate autopilot;
extern crate image;
use std::fs::File;
use std::path::Path;

fn main() {
    let bmp = autopilot::bitmap::capture_screen().expect("Unable to capture screen");
    let path = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test.png");
    let ref mut fout = File::create(&path).expect("Unable to create output file");
    let _ = bmp.image
        .save(fout, image::PNG)
        .expect("Unable to save image");
    println!("Scale factor {}", autopilot::screen::scale());
    println!("Screen size {}", autopilot::screen::size());
    println!("Saved screenshot at {}", path.to_str().unwrap_or(""));
}
