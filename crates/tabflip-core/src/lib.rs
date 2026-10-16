//! tabflip's core: everything that needs neither a window nor the OS.
//!
//! The app (`tabflip-app`) feeds this crate what it reads from macOS and
//! does what it is told back. Every rule about which window comes first,
//! what a row says and what a key does lives here, so it can be tested
//! without a Mac.

pub mod config;
pub mod label;
pub mod look;
pub mod order;
pub mod shortcut;
pub mod switcher;
pub mod update;
