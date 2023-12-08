//! # egui-keybind, a hotkey library for [egui](https://github.com/emilk/egui)
//!
//! This library provides a simple [egui](https://github.com/emilk/egui) widget for keybindings (hotkeys).
//!
//! # Serde
//!
//! For serde support, enable the `serde` feature.
//!
//! # License
//!
//! Public domain or MIT or Boost Software License

#![warn(missing_docs)]

mod bind;
mod keybind;
pub use bind::*;
pub use keybind::*;
