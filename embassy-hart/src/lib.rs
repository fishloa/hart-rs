//! Async HART master for Embassy.
//!
//! Combines the `ad5700` async modem driver with `hart-protocol` codec
//! and `embassy-time` timeouts to provide a typed async HART master API.

#![no_std]

pub mod master;
