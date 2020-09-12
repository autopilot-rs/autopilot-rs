extern crate pkg_config;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "linux" {
        println!("cargo:rustc-flags=-l X11 -l Xtst");
    }
}
