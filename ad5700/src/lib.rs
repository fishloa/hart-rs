//! AD5700-1 HART modem driver for embedded-hal.
//!
//! Provides blocking and async drivers for the Analog Devices AD5700-1
//! HART modem, plus a blocking HART master that combines the modem
//! driver with the `hart-protocol` codec.

#![no_std]

/// The version of this crate, set at compile time from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod asynch;
pub mod blocking;
pub mod error;
pub mod master;
