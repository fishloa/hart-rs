//! HART protocol codec for embedded systems.
//!
//! This crate provides `no_std` encode/decode support for the HART
//! (Highway Addressable Remote Transducer) protocol, including frame
//! encoding, byte-at-a-time decoding, typed command request/response
//! structs, and engineering unit codes.
//!
//! No I/O is performed — this is a pure codec library.

#![no_std]

/// The version of this crate, set at compile time from Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod commands;
pub mod consts;
pub mod decode;
pub mod encode;
pub mod error;
pub mod packed_string;
pub mod types;
pub mod units;
