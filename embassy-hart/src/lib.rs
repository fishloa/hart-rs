//! Async HART master for Embassy.
//!
//! Combines the `ad5700` async modem driver with `hart-protocol` codec
//! and `embassy-time` timeouts to provide a typed async HART master API.

#![no_std]

/// The version of this crate, set at compile time from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod master;
