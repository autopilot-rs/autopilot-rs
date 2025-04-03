fn main() {
    #[cfg(target_os = "linux")]
    {
        pkg_config::Config::new().atleast_version("1").probe("x11").unwrap();
        pkg_config::Config::new().atleast_version("1").probe("xtst").unwrap();
    }
}
