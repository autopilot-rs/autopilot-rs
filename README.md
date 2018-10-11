[![Docs](https://docs.rs/autopilot/badge.svg)](https://docs.rs/autopilot)
[![Crates.io](https://img.shields.io/crates/v/autopilot.svg)](https://crates.io/crates/autopilot)
[![Travis Build Status](https://travis-ci.org/autopilot-rs/autopilot-rs.svg?branch=master)](https://travis-ci.org/autopilot-rs/autopilot-rs)
[![Appveyor Build Status](https://ci.appveyor.com/api/projects/status/ilcq8ev8ht49eqbx?svg=true)](https://ci.appveyor.com/project/msanders/autopilot-rs)

# AutoPilot

AutoPilot is a Rust port of the Python C extension
[AutoPy](http://autopy.org), a simple, cross-platform GUI automation library for
Python. For more information, see the
[README](https://github.com/autopilot-rs/autopy#autopy-introduction-and-tutorial)
on that repo.

Currently supported on macOS, Windows, and X11 with the XTest extension.

## Examples

The following will move the mouse across the screen as a sine wave:

```rust
extern crate autopilot;
extern crate rand;
use rand::Rng;

const TWO_PI: f64 = std::f64::consts::PI * 2.0;
fn sine_mouse_wave() {
    let screen_size = autopilot::screen::size();
    let scoped_height = screen_size.height / 2.0 - 10.0; // Stay in screen bounds.
    let rng = rand::thread_rng();
    for x in 0..screen_size.width as u64 {
        let y = (scoped_height * ((TWO_PI * x as f64) / screen_size.width).sin() + 
                 scoped_height).round();
        let duration: u64 = rng.gen_range(1, 3);
        autopilot::mouse::move_to(autopilot::geometry::Point::new(
            x as f64,
            y as f64
        )).expect("Unable to move mouse");
        std::thread::sleep(std::time::Duration::from_millis(duration));
    }
}
```

This will enter the keys from the string "Hello, world!" and then prompt an alert with the same text:

```rust
extern crate autopilot;

fn main() {
    autopilot::key::type_string("Hello, world!", &[], 200., 0.);
    let _ = autopilot::alert::alert("Hello, world!", None, None, None);
}
```
