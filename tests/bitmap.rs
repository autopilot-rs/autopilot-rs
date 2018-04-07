extern crate autopilot;
extern crate image;
use autopilot::bitmap::Bitmap;
use std::path;

#[test]
fn find_bitmap() {
    let haystack_path = asset_path().join("haystack.png");
    let haystack = Bitmap::new(image::open(haystack_path).unwrap(), None);
    for idx in 0..2 {
        let needle_path = asset_path().join(format!("needle{}.png", idx + 1));
        let needle = Bitmap::new(image::open(needle_path).unwrap(), None);
        let pt = haystack.find_bitmap(&needle, None, None, None);
        assert_eq!(pt.is_some(), true);
    }
}

#[inline]
fn asset_path() -> path::PathBuf {
    path::Path::new(file!()).parent().unwrap().join("assets")
}
