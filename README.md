# autopilot-rs

autopilot-rs is a Rust port of the Python C extension
[AutoPy](http://autopy.org), a simple, cross-platform GUI automation library for
Python. For more information, see the
[README](https://github.com/autopilot-rs/autopy#autopy-introduction-and-tutorial)
on that repo.

Currently supported on macOS, Windows, and X11 with the XTest extension.

## Examples

```rust
extern crate autopilot;
extern crate rand;
use rand::Rng;

const TWO_PI: f64 = std::f64::consts::PI * 2.0;
fn sine_mouse_wave() {
    let screen_size = autopilot::screen::size();
    let scoped_height = screen_size.height / 2.0 - 10.0; // Stay in screen bounds.
    for x in 0..screen_size.width as u64 {
        let y = (scoped_height * ((TWO_PI * x as f64) / screen_size.width).sin() + 
                 scoped_height).round();
        let duration: u64 = rand::thread_rng().gen_range(1, 3);
        autopilot::mouse::move_to(autopilot::geometry::Point::new(
            x as f64,
            y as f64
        )).expect("Unable to move mouse");
        std::thread::sleep(std::time::Duration::from_millis(duration));
    }
}
```

```rust
extern crate autopilot;

fn main() {
    autopilot::key::type_string("Hello, world!", None, None, &[]);
    let _ = autopilot::alert::alert("Hello, world!", None, None, None);
}
```
