# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.1 - 2019-07-12

### Fixed
- Added explicit panic for when X11 is not running.
- Fixed compilation on certain devices, e.g. Raspberry Pi.

## 0.2.0 - 2019-05-11

### Changed
- Updated image, libc, scopeguard, quickcheck, pkg-config, core-foundation,
  core-graphics, and cocoa crates to latest versions.

## 0.1.9 - 2019-05-10

### Fixed
- Fixed various warnings from clippy linter.

### Changed
- Updated image crate to 0.20.

## 0.1.8 - 2018-08-26

### Fixed
- Updated scale factor on x11 to be rounded.

## 0.1.7 - 2018-08-09

### Changed
- Updated Cocoa and other macOS dependencies.
- Updated x11 dependency.

### Fixed
- Fixed compilation error on 32-bit Linux.

## 0.1.6 - 2018-08-06

### Fixed
- Fixed compilation error on 32-bit Windows.

## 0.1.5 - 2018-08-02

### Added
- Added constant for spacebar key.

### Fixed
- Fixed linux arrow keycode constant definitions.
- Fixed colon showing up as semicolon on Windows.

## 0.1.4 - 2018-06-02

### Fixed
- Fixed compilation error on Windows for mouse scroll events.

### Changed
- Update function signatures with `delay` parameters to be consistent.

## 0.1.3 - 2018-03-27

### Added
- Added support for mouse scroll events via `mouse::scroll`.
- Added support for passing a delay to `mouse::click`.

### Fixed
- Fixed `mouse::click` to release at end of function.
- Updated `key::tap` delay to be passed through to modifier key toggles.

## 0.1.2 - 2018-05-12

### Added
- Added `KeyCode::Tab` constant.
- Added support for passing delay into `key::tap`.
- Added support faster typing with `key::type_string`.

### Changed
- Updated Cocoa and other macOS dependencies.
- Updated `mouse::smooth_move` to accept a duration.
- Updated `key::type_string` delay parameters from `Option<f64>` to `f64`.

## 0.1.1 - 2018-04-30

### Added
- Implemented `Hash` for `Bitmap`, `Size` and `Point`.

## 0.1.0 - 2018-04-30
- Initial release.
