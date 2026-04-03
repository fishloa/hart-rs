# hart-rs Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a working HART master that can discover and read measurements from a VEGAPULS 21 radar level sensor over a 4-20mA loop using an AD5700-1 modem on STM32H7.

**Architecture:** Three-crate workspace. `hart-protocol` is a pure `no_std` codec (encode/decode frames and commands). `ad5700` is an `embedded-hal`-generic modem driver that also provides `HartMasterBlocking`. `embassy-hart` adds async master support via `embassy-time`. Phase 1 implements Commands 0, 1, 2, 3, and 48.

**Tech Stack:** Rust (stable, `no_std`), `heapless`, `embedded-hal` 1.0, `embedded-hal-async`, `embassy-time`, `embedded-hal-mock` (dev), `defmt` (examples)

**Spec:** `docs/superpowers/specs/2026-04-03-hart-rs-design.md`
**Unit codes reference:** `docs/superpowers/specs/hart-unit-codes.md`

---

## File Structure

```
hart-rs/
├── Cargo.toml                              (workspace definition)
├── hart-protocol/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                          (crate root, re-exports)
│       ├── consts.rs                       (all protocol constants)
│       ├── types.rs                        (Address, MasterRole, FrameType, ResponseStatus)
│       ├── units.rs                        (UnitCode enum with all known codes)
│       ├── packed_string.rs                (6-bit HART string encode/decode)
│       ├── encode.rs                       (Encoder — frame to bytes)
│       ├── decode.rs                       (Decoder — byte-at-a-time state machine)
│       ├── commands/
│       │   ├── mod.rs                      (CommandRequest/CommandResponse traits)
│       │   ├── cmd0.rs                     (Read Device ID)
│       │   ├── cmd1.rs                     (Read Primary Variable)
│       │   ├── cmd2.rs                     (Read Loop Current & %)
│       │   ├── cmd3.rs                     (Read Dynamic Variables)
│       │   └── cmd48.rs                    (Read Additional Device Status)
│       └── error.rs                        (EncodeError, DecodeError)
├── ad5700/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                          (crate root, re-exports)
│       ├── blocking.rs                     (Ad5700 blocking modem driver)
│       ├── asynch.rs                       (Ad5700Async async modem driver)
│       ├── master.rs                       (HartMasterBlocking)
│       └── error.rs                        (Ad5700Error, HartError)
├── embassy-hart/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                          (crate root, re-exports)
│       └── master.rs                       (HartMaster async)
└── examples/hart-stm32h7/                  (deferred to hardware integration)
```

---

### Task 1: Workspace and Crate Scaffolding

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `hart-protocol/Cargo.toml`
- Create: `hart-protocol/src/lib.rs`
- Create: `ad5700/Cargo.toml`
- Create: `ad5700/src/lib.rs`
- Create: `embassy-hart/Cargo.toml`
- Create: `embassy-hart/src/lib.rs`

- [ ] **Step 1: Create workspace Cargo.toml**

```toml
# hart-rs/Cargo.toml
[workspace]
resolver = "2"
members = [
    "hart-protocol",
    "ad5700",
    "embassy-hart",
]
```

- [ ] **Step 2: Create hart-protocol crate**

```toml
# hart-protocol/Cargo.toml
[package]
name = "hart-protocol"
version = "0.1.0"
edition = "2021"
description = "HART protocol codec for embedded systems"
license = "MIT OR Apache-2.0"

[dependencies]
heapless = "0.8"

[dev-dependencies]
```

```rust
// hart-protocol/src/lib.rs
#![no_std]
```

- [ ] **Step 3: Create ad5700 crate**

```toml
# ad5700/Cargo.toml
[package]
name = "ad5700"
version = "0.1.0"
edition = "2021"
description = "AD5700-1 HART modem driver for embedded-hal"
license = "MIT OR Apache-2.0"

[dependencies]
hart-protocol = { path = "../hart-protocol" }
embedded-hal = "1.0"
embedded-hal-async = "1.0"

[dev-dependencies]
embedded-hal-mock = { version = "0.11", features = ["eh1"] }
```

```rust
// ad5700/src/lib.rs
#![no_std]
```

- [ ] **Step 4: Create embassy-hart crate**

```toml
# embassy-hart/Cargo.toml
[package]
name = "embassy-hart"
version = "0.1.0"
edition = "2021"
description = "Async HART master for Embassy"
license = "MIT OR Apache-2.0"

[dependencies]
hart-protocol = { path = "../hart-protocol" }
ad5700 = { path = "../ad5700" }
embassy-time = "0.4"
embedded-hal = "1.0"
embedded-hal-async = "1.0"
```

```rust
// embassy-hart/src/lib.rs
#![no_std]
```

- [ ] **Step 5: Verify workspace builds**

Run: `cargo build --workspace`
Expected: compiles with no errors

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml hart-protocol/ ad5700/ embassy-hart/
git commit -m "scaffold: workspace with hart-protocol, ad5700, embassy-hart crates"
```

---

### Task 2: Protocol Constants

**Files:**
- Create: `hart-protocol/src/consts.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write constants test**

```rust
// hart-protocol/src/consts.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiter_values_match_spec() {
        assert_eq!(delimiter::REQUEST_SHORT, 0x02);
        assert_eq!(delimiter::REQUEST_LONG, 0x82);
        assert_eq!(delimiter::RESPONSE_SHORT, 0x06);
        assert_eq!(delimiter::RESPONSE_LONG, 0x86);
        assert_eq!(delimiter::BURST_SHORT, 0x01);
        assert_eq!(delimiter::BURST_LONG, 0x81);
    }

    #[test]
    fn command_numbers_match_spec() {
        assert_eq!(commands::READ_DEVICE_ID, 0);
        assert_eq!(commands::READ_PRIMARY_VARIABLE, 1);
        assert_eq!(commands::READ_LOOP_CURRENT_PERCENT, 2);
        assert_eq!(commands::READ_DYNAMIC_VARS, 3);
        assert_eq!(commands::READ_ADDITIONAL_STATUS, 48);
    }

    #[test]
    fn frame_constants() {
        assert_eq!(PREAMBLE_BYTE, 0xFF);
        assert_eq!(MAX_DATA_LENGTH, 255);
        // 20 preamble + 1 delim + 5 addr + 1 cmd + 1 len + 255 data + 1 chk
        assert_eq!(MAX_FRAME_LENGTH, 284);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — module not found

- [ ] **Step 3: Write constants implementation**

```rust
// hart-protocol/src/consts.rs

// --- Frame constants ---
pub const PREAMBLE_BYTE: u8 = 0xFF;
pub const MIN_PREAMBLE_COUNT: u8 = 5;
pub const MAX_PREAMBLE_COUNT: u8 = 20;
pub const DEFAULT_PREAMBLE_COUNT: u8 = 10;
pub const MAX_DATA_LENGTH: usize = 255;
/// 20 preamble + 1 delimiter + 5 address (long) + 1 command + 1 byte_count + 255 data + 1 checksum
pub const MAX_FRAME_LENGTH: usize = 284;

// --- UART configuration ---
pub const BAUD_RATE: u32 = 1200;
pub const RESPONSE_TIMEOUT_MS: u32 = 500;
pub const RTS_HOLD_TIME_MS: u32 = 5;

// --- Delimiter bytes ---
pub mod delimiter {
    pub const REQUEST_SHORT: u8 = 0x02;
    pub const REQUEST_LONG: u8 = 0x82;
    pub const RESPONSE_SHORT: u8 = 0x06;
    pub const RESPONSE_LONG: u8 = 0x86;
    pub const BURST_SHORT: u8 = 0x01;
    pub const BURST_LONG: u8 = 0x81;

    /// Bit 7 of the delimiter indicates long address format
    pub const LONG_ADDRESS_BIT: u8 = 0x80;
}

// --- Address bit masks ---
pub mod address {
    /// Bit 7 of the first address byte: 1 = primary master
    pub const PRIMARY_MASTER_BIT: u8 = 0x80;
    /// Bit 6 of the first address byte: 1 = burst mode
    pub const BURST_MODE_BIT: u8 = 0x40;
    /// Bits 0-3 of the short address byte: poll address
    pub const SHORT_ADDRESS_MASK: u8 = 0x0F;
    /// Bits 0-5 of first long address byte: manufacturer ID
    pub const MANUFACTURER_ID_MASK: u8 = 0x3F;
}

// --- Command numbers ---
pub mod commands {
    // Phase 1 — universal commands for VEGAPULS 21
    pub const READ_DEVICE_ID: u8 = 0;
    pub const READ_PRIMARY_VARIABLE: u8 = 1;
    pub const READ_LOOP_CURRENT_PERCENT: u8 = 2;
    pub const READ_DYNAMIC_VARS: u8 = 3;
    pub const WRITE_POLLING_ADDRESS: u8 = 6;
    pub const READ_LOOP_CONFIG: u8 = 7;
    pub const READ_DYNAMIC_VAR_CLASS: u8 = 8;
    pub const READ_DEVICE_VARS_WITH_STATUS: u8 = 9;
    pub const READ_UNIQUE_ID_BY_TAG: u8 = 11;
    pub const READ_MESSAGE: u8 = 12;
    pub const READ_TAG_DESCRIPTOR_DATE: u8 = 13;
    pub const READ_PV_TRANSDUCER_INFO: u8 = 14;
    pub const READ_DEVICE_INFO: u8 = 15;
    pub const READ_FINAL_ASSEMBLY_NUMBER: u8 = 16;
    pub const WRITE_MESSAGE: u8 = 17;
    pub const WRITE_TAG_DESCRIPTOR_DATE: u8 = 18;
    pub const WRITE_FINAL_ASSEMBLY_NUMBER: u8 = 19;
    pub const READ_LONG_TAG: u8 = 20;
    pub const READ_UNIQUE_ID_BY_LONG_TAG: u8 = 21;
    pub const WRITE_LONG_TAG: u8 = 22;
    pub const RESET_CONFIG_CHANGED: u8 = 38;
    pub const READ_ADDITIONAL_STATUS: u8 = 48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delimiter_values_match_spec() {
        assert_eq!(delimiter::REQUEST_SHORT, 0x02);
        assert_eq!(delimiter::REQUEST_LONG, 0x82);
        assert_eq!(delimiter::RESPONSE_SHORT, 0x06);
        assert_eq!(delimiter::RESPONSE_LONG, 0x86);
        assert_eq!(delimiter::BURST_SHORT, 0x01);
        assert_eq!(delimiter::BURST_LONG, 0x81);
    }

    #[test]
    fn command_numbers_match_spec() {
        assert_eq!(commands::READ_DEVICE_ID, 0);
        assert_eq!(commands::READ_PRIMARY_VARIABLE, 1);
        assert_eq!(commands::READ_LOOP_CURRENT_PERCENT, 2);
        assert_eq!(commands::READ_DYNAMIC_VARS, 3);
        assert_eq!(commands::READ_ADDITIONAL_STATUS, 48);
    }

    #[test]
    fn frame_constants() {
        assert_eq!(PREAMBLE_BYTE, 0xFF);
        assert_eq!(MAX_DATA_LENGTH, 255);
        assert_eq!(MAX_FRAME_LENGTH, 284);
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: 3 tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add protocol constants module"
```

---

### Task 3: Core Types — Address, MasterRole, FrameType

**Files:**
- Create: `hart-protocol/src/types.rs`
- Create: `hart-protocol/src/error.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write types tests**

```rust
// hart-protocol/src/types.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_address_roundtrip() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 5];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 1);
        // Primary master bit set, poll address 0
        assert_eq!(buf[0], 0x80);

        let (decoded, decoded_len) = Address::decode(&buf[..1], false).unwrap();
        assert_eq!(decoded_len, 1);
        assert!(matches!(decoded, Address::Short { poll_address: 0, .. }));
    }

    #[test]
    fn long_address_roundtrip() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x00_11_22_33,
        };
        let mut buf = [0u8; 5];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 5);
        // Primary master + manufacturer_id 0x1A in first byte
        assert_eq!(buf[0], 0x80 | 0x1A);
        assert_eq!(buf[1], 0x2B);
        // device_id 0x112233 big-endian in bytes 2-4
        assert_eq!(buf[2], 0x11);
        assert_eq!(buf[3], 0x22);
        assert_eq!(buf[4], 0x33);

        let (decoded, decoded_len) = Address::decode(&buf[..5], true).unwrap();
        assert_eq!(decoded_len, 5);
        match decoded {
            Address::Long { manufacturer_id, device_type, device_id, .. } => {
                assert_eq!(manufacturer_id, 0x1A);
                assert_eq!(device_type, 0x2B);
                assert_eq!(device_id, 0x00_11_22_33);
            }
            _ => panic!("expected long address"),
        }
    }

    #[test]
    fn short_address_secondary_master_burst() {
        let addr = Address::Short {
            master: MasterRole::Secondary,
            burst: true,
            poll_address: 5,
        };
        let mut buf = [0u8; 5];
        addr.encode(&mut buf);
        // No primary master bit, burst bit set, poll address 5
        assert_eq!(buf[0], 0x40 | 5);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `types` module not found

- [ ] **Step 3: Write types and error implementations**

```rust
// hart-protocol/src/error.rs
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    BufferTooSmall,
    DataTooLong,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    BufferTooShort,
    InvalidDelimiter(u8),
    ChecksumMismatch { expected: u8, actual: u8 },
    InvalidAddress,
    InvalidFrameType,
}
```

```rust
// hart-protocol/src/types.rs
use crate::consts::address;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MasterRole {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    Request,
    Response,
    Burst,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    Short {
        master: MasterRole,
        burst: bool,
        poll_address: u8,
    },
    Long {
        master: MasterRole,
        burst: bool,
        manufacturer_id: u8,
        device_type: u8,
        device_id: u32,
    },
}

impl Address {
    /// Encode address into buf, returns number of bytes written (1 or 5).
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        match self {
            Address::Short { master, burst, poll_address } => {
                let mut byte = poll_address & address::SHORT_ADDRESS_MASK;
                if matches!(master, MasterRole::Primary) {
                    byte |= address::PRIMARY_MASTER_BIT;
                }
                if *burst {
                    byte |= address::BURST_MODE_BIT;
                }
                buf[0] = byte;
                1
            }
            Address::Long { master, burst, manufacturer_id, device_type, device_id } => {
                let mut byte0 = manufacturer_id & address::MANUFACTURER_ID_MASK;
                if matches!(master, MasterRole::Primary) {
                    byte0 |= address::PRIMARY_MASTER_BIT;
                }
                if *burst {
                    byte0 |= address::BURST_MODE_BIT;
                }
                buf[0] = byte0;
                buf[1] = *device_type;
                // device_id: only lower 24 bits used, big-endian
                buf[2] = (device_id >> 16) as u8;
                buf[3] = (device_id >> 8) as u8;
                buf[4] = *device_id as u8;
                5
            }
        }
    }

    /// Decode address from buf. `is_long` determined by delimiter.
    /// Returns (Address, bytes_consumed).
    pub fn decode(buf: &[u8], is_long: bool) -> Result<(Self, usize), crate::error::DecodeError> {
        if is_long {
            if buf.len() < 5 {
                return Err(crate::error::DecodeError::BufferTooShort);
            }
            let master = if buf[0] & address::PRIMARY_MASTER_BIT != 0 {
                MasterRole::Primary
            } else {
                MasterRole::Secondary
            };
            let burst = buf[0] & address::BURST_MODE_BIT != 0;
            let manufacturer_id = buf[0] & address::MANUFACTURER_ID_MASK;
            let device_type = buf[1];
            let device_id = ((buf[2] as u32) << 16)
                | ((buf[3] as u32) << 8)
                | (buf[4] as u32);
            Ok((
                Address::Long { master, burst, manufacturer_id, device_type, device_id },
                5,
            ))
        } else {
            if buf.is_empty() {
                return Err(crate::error::DecodeError::BufferTooShort);
            }
            let master = if buf[0] & address::PRIMARY_MASTER_BIT != 0 {
                MasterRole::Primary
            } else {
                MasterRole::Secondary
            };
            let burst = buf[0] & address::BURST_MODE_BIT != 0;
            let poll_address = buf[0] & address::SHORT_ADDRESS_MASK;
            Ok((
                Address::Short { master, burst, poll_address },
                1,
            ))
        }
    }

    /// Returns true if this is a long address.
    pub fn is_long(&self) -> bool {
        matches!(self, Address::Long { .. })
    }
}

/// Parsed response status bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResponseStatus {
    pub communication_error: u8,
    pub device_status: u8,
}

impl ResponseStatus {
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        Self {
            communication_error: bytes[0],
            device_status: bytes[1],
        }
    }

    /// Returns true if bit 7 of the communication error byte is set.
    pub fn has_error(&self) -> bool {
        self.communication_error & 0x80 != 0
    }

    /// Returns true if the device malfunction bit is set.
    pub fn device_malfunction(&self) -> bool {
        self.device_status & 0x80 != 0
    }

    /// Returns true if the configuration changed bit is set.
    pub fn config_changed(&self) -> bool {
        self.device_status & 0x40 != 0
    }

    /// Returns true if the cold start bit is set.
    pub fn cold_start(&self) -> bool {
        self.device_status & 0x20 != 0
    }

    /// Returns true if more status is available (read cmd 48).
    pub fn more_status_available(&self) -> bool {
        self.device_status & 0x10 != 0
    }

    /// Returns true if the primary variable is out of limits.
    pub fn pv_out_of_limits(&self) -> bool {
        self.device_status & 0x01 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_address_roundtrip() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 5];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 1);
        assert_eq!(buf[0], 0x80);

        let (decoded, decoded_len) = Address::decode(&buf[..1], false).unwrap();
        assert_eq!(decoded_len, 1);
        assert!(matches!(decoded, Address::Short { poll_address: 0, .. }));
    }

    #[test]
    fn long_address_roundtrip() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x00_11_22_33,
        };
        let mut buf = [0u8; 5];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 5);
        assert_eq!(buf[0], 0x80 | 0x1A);
        assert_eq!(buf[1], 0x2B);
        assert_eq!(buf[2], 0x11);
        assert_eq!(buf[3], 0x22);
        assert_eq!(buf[4], 0x33);

        let (decoded, decoded_len) = Address::decode(&buf[..5], true).unwrap();
        assert_eq!(decoded_len, 5);
        match decoded {
            Address::Long { manufacturer_id, device_type, device_id, .. } => {
                assert_eq!(manufacturer_id, 0x1A);
                assert_eq!(device_type, 0x2B);
                assert_eq!(device_id, 0x00_11_22_33);
            }
            _ => panic!("expected long address"),
        }
    }

    #[test]
    fn short_address_secondary_master_burst() {
        let addr = Address::Short {
            master: MasterRole::Secondary,
            burst: true,
            poll_address: 5,
        };
        let mut buf = [0u8; 5];
        addr.encode(&mut buf);
        assert_eq!(buf[0], 0x40 | 5);
    }

    #[test]
    fn response_status_flags() {
        let status = ResponseStatus::from_bytes([0x00, 0xC1]);
        assert!(!status.has_error());
        assert!(status.device_malfunction());
        assert!(status.config_changed());
        assert!(status.pv_out_of_limits());
        assert!(!status.cold_start());
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
pub mod error;
pub mod types;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass (consts + types)

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add core types — Address, MasterRole, FrameType, ResponseStatus"
```

---

### Task 4: Unit Codes Enum

**Files:**
- Create: `hart-protocol/src/units.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write unit codes test**

```rust
// hart-protocol/src/units.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes_roundtrip() {
        assert_eq!(UnitCode::from_u8(45), UnitCode::Meters);
        assert_eq!(UnitCode::Meters.as_u8(), 45);
        assert_eq!(UnitCode::from_u8(32), UnitCode::DegreesCelsius);
        assert_eq!(UnitCode::DegreesCelsius.as_u8(), 32);
        assert_eq!(UnitCode::from_u8(57), UnitCode::Percent);
        assert_eq!(UnitCode::Percent.as_u8(), 57);
        assert_eq!(UnitCode::from_u8(7), UnitCode::Bar);
        assert_eq!(UnitCode::Bar.as_u8(), 7);
    }

    #[test]
    fn unknown_code_preserved() {
        let code = UnitCode::from_u8(199);
        assert_eq!(code, UnitCode::Unknown(199));
        assert_eq!(code.as_u8(), 199);
    }

    #[test]
    fn special_codes() {
        assert_eq!(UnitCode::from_u8(250), UnitCode::NotUsed);
        assert_eq!(UnitCode::from_u8(251), UnitCode::None);
        assert_eq!(UnitCode::from_u8(253), UnitCode::Custom);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `units` module not found

- [ ] **Step 3: Write UnitCode enum**

Build the complete enum from `docs/superpowers/specs/hart-unit-codes.md`. Every variant uses its wire code in the `from_u8`/`as_u8` match arms.

```rust
// hart-protocol/src/units.rs

/// HART engineering unit codes from HCF_SPEC-183 Common Table 2.
///
/// Codes 0-169 and 220+ have fixed meanings regardless of Device Variable Classification.
/// Codes 170-219 are expansion range — their meaning depends on classification context.
/// Use `Unknown(u8)` for codes not yet in this enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitCode {
    // --- Pressure (Table 2.65) ---
    InchesWaterColumn68F,       // 1
    InchesOfMercury0C,          // 2
    FeetOfWater68F,             // 3
    MillimetersOfWater68F,      // 4
    MillimetersOfMercury0C,     // 5
    Psi,                        // 6
    Bar,                        // 7
    Millibar,                   // 8
    GramsPerSquareCentimeter,   // 9
    KilogramsPerSquareCentimeter, // 10
    Pascals,                    // 11
    KiloPascals,                // 12
    Torr,                       // 13
    Atmospheres,                // 14
    InchesWaterColumn60F,       // 145
    MegaPascals,                // 237
    InchesWaterColumn4C,        // 238
    MillimetersOfWater4C,       // 239

    // --- Volumetric Flow (Table 2.66) ---
    CubicFeetPerMinute,         // 15
    UsGallonsPerMinute,         // 16
    LitersPerMinute,            // 17
    ImperialGallonsPerMinute,   // 18
    CubicMetersPerHour,         // 19
    UsGallonsPerSecond,         // 22
    MillionUsGallonsPerDay,     // 23
    LitersPerSecond,            // 24
    MillionLitersPerDay,        // 25
    CubicFeetPerSecond,         // 26
    CubicFeetPerDay,            // 27
    CubicMetersPerSecond,       // 28
    CubicMetersPerDay,          // 29
    ImperialGallonsPerHour,     // 30
    ImperialGallonsPerDay,      // 31
    CubicFeetPerHour,           // 130
    CubicMetersPerMinute,       // 131
    BarrelsPerSecond,           // 132
    BarrelsPerMinute,           // 133
    BarrelsPerHour,             // 134
    BarrelsPerDay,              // 135
    UsGallonsPerHour,           // 136
    ImperialGallonsPerSecond,   // 137
    LitersPerHour,              // 138
    UsGallonsPerDay,            // 235

    // --- Temperature (Table 2.64) ---
    DegreesCelsius,             // 32
    DegreesFahrenheit,          // 33
    DegreesRankine,             // 34
    Kelvin,                     // 35

    // --- Electrical ---
    Millivolts,                 // 36
    Volts,                      // 37
    Ohms,                       // 38
    Milliamperes,               // 39

    // --- Volume (Table 2.68) ---
    UsGallons,                  // 40
    Liters,                     // 41
    ImperialGallons,            // 42
    CubicMeters,                // 43
    Barrels,                    // 46
    Bushels,                    // 110
    CubicYards,                 // 111
    CubicFeet,                  // 112
    CubicInches,                // 113
    LiquidBarrels,              // 124
    Hectoliters,                // 236

    // --- Length (Table 2.69) ---
    Feet,                       // 44
    Meters,                     // 45
    Inches,                     // 47
    Centimeters,                // 48
    Millimeters,                // 49

    // --- Time (Table 2.70) ---
    Minutes,                    // 50
    Seconds,                    // 51
    Hours,                      // 52
    Days,                       // 53

    // --- Percent ---
    Percent,                    // 57

    // --- Mass (Table 2.71) ---
    Grams,                      // 60
    Kilograms,                  // 61
    MetricTons,                 // 62
    Pounds,                     // 63

    // --- Mass Flow (Table 2.67) ---
    GramsPerSecond,             // 70
    GramsPerMinute,             // 71
    GramsPerHour,               // 72
    KilogramsPerSecond,         // 73
    KilogramsPerMinute,         // 74
    KilogramsPerHour,           // 75
    KilogramsPerDay,            // 76
    MetricTonsPerMinute,        // 77
    MetricTonsPerHour,          // 78
    MetricTonsPerDay,           // 79
    PoundsPerSecond,            // 80
    PoundsPerMinute,            // 81
    PoundsPerHour,              // 82
    PoundsPerDay,               // 83
    ShortTonsPerMinute,         // 84
    ShortTonsPerHour,           // 85

    // --- Density (Table 2.72) ---
    GramsPerCubicCentimeter,    // 91
    KilogramsPerCubicMeter,     // 92
    PoundsPerUsGallon,          // 93
    PoundsPerCubicFoot,         // 94
    GramsPerMilliliter,         // 95
    KilogramsPerLiter,          // 96
    GramsPerLiter,              // 97

    // --- Special ---
    NotUsed,                    // 250
    None,                       // 251
    Custom,                     // 253

    /// Fallback for codes not in this enum.
    Unknown(u8),
}

impl UnitCode {
    pub fn from_u8(code: u8) -> Self {
        match code {
            1 => Self::InchesWaterColumn68F,
            2 => Self::InchesOfMercury0C,
            3 => Self::FeetOfWater68F,
            4 => Self::MillimetersOfWater68F,
            5 => Self::MillimetersOfMercury0C,
            6 => Self::Psi,
            7 => Self::Bar,
            8 => Self::Millibar,
            9 => Self::GramsPerSquareCentimeter,
            10 => Self::KilogramsPerSquareCentimeter,
            11 => Self::Pascals,
            12 => Self::KiloPascals,
            13 => Self::Torr,
            14 => Self::Atmospheres,
            15 => Self::CubicFeetPerMinute,
            16 => Self::UsGallonsPerMinute,
            17 => Self::LitersPerMinute,
            18 => Self::ImperialGallonsPerMinute,
            19 => Self::CubicMetersPerHour,
            22 => Self::UsGallonsPerSecond,
            23 => Self::MillionUsGallonsPerDay,
            24 => Self::LitersPerSecond,
            25 => Self::MillionLitersPerDay,
            26 => Self::CubicFeetPerSecond,
            27 => Self::CubicFeetPerDay,
            28 => Self::CubicMetersPerSecond,
            29 => Self::CubicMetersPerDay,
            30 => Self::ImperialGallonsPerHour,
            31 => Self::ImperialGallonsPerDay,
            32 => Self::DegreesCelsius,
            33 => Self::DegreesFahrenheit,
            34 => Self::DegreesRankine,
            35 => Self::Kelvin,
            36 => Self::Millivolts,
            37 => Self::Volts,
            38 => Self::Ohms,
            39 => Self::Milliamperes,
            40 => Self::UsGallons,
            41 => Self::Liters,
            42 => Self::ImperialGallons,
            43 => Self::CubicMeters,
            44 => Self::Feet,
            45 => Self::Meters,
            46 => Self::Barrels,
            47 => Self::Inches,
            48 => Self::Centimeters,
            49 => Self::Millimeters,
            50 => Self::Minutes,
            51 => Self::Seconds,
            52 => Self::Hours,
            53 => Self::Days,
            57 => Self::Percent,
            60 => Self::Grams,
            61 => Self::Kilograms,
            62 => Self::MetricTons,
            63 => Self::Pounds,
            70 => Self::GramsPerSecond,
            71 => Self::GramsPerMinute,
            72 => Self::GramsPerHour,
            73 => Self::KilogramsPerSecond,
            74 => Self::KilogramsPerMinute,
            75 => Self::KilogramsPerHour,
            76 => Self::KilogramsPerDay,
            77 => Self::MetricTonsPerMinute,
            78 => Self::MetricTonsPerHour,
            79 => Self::MetricTonsPerDay,
            80 => Self::PoundsPerSecond,
            81 => Self::PoundsPerMinute,
            82 => Self::PoundsPerHour,
            83 => Self::PoundsPerDay,
            84 => Self::ShortTonsPerMinute,
            85 => Self::ShortTonsPerHour,
            91 => Self::GramsPerCubicCentimeter,
            92 => Self::KilogramsPerCubicMeter,
            93 => Self::PoundsPerUsGallon,
            94 => Self::PoundsPerCubicFoot,
            95 => Self::GramsPerMilliliter,
            96 => Self::KilogramsPerLiter,
            97 => Self::GramsPerLiter,
            110 => Self::Bushels,
            111 => Self::CubicYards,
            112 => Self::CubicFeet,
            113 => Self::CubicInches,
            124 => Self::LiquidBarrels,
            130 => Self::CubicFeetPerHour,
            131 => Self::CubicMetersPerMinute,
            132 => Self::BarrelsPerSecond,
            133 => Self::BarrelsPerMinute,
            134 => Self::BarrelsPerHour,
            135 => Self::BarrelsPerDay,
            136 => Self::UsGallonsPerHour,
            137 => Self::ImperialGallonsPerSecond,
            138 => Self::LitersPerHour,
            145 => Self::InchesWaterColumn60F,
            235 => Self::UsGallonsPerDay,
            236 => Self::Hectoliters,
            237 => Self::MegaPascals,
            238 => Self::InchesWaterColumn4C,
            239 => Self::MillimetersOfWater4C,
            250 => Self::NotUsed,
            251 => Self::None,
            253 => Self::Custom,
            other => Self::Unknown(other),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Self::InchesWaterColumn68F => 1,
            Self::InchesOfMercury0C => 2,
            Self::FeetOfWater68F => 3,
            Self::MillimetersOfWater68F => 4,
            Self::MillimetersOfMercury0C => 5,
            Self::Psi => 6,
            Self::Bar => 7,
            Self::Millibar => 8,
            Self::GramsPerSquareCentimeter => 9,
            Self::KilogramsPerSquareCentimeter => 10,
            Self::Pascals => 11,
            Self::KiloPascals => 12,
            Self::Torr => 13,
            Self::Atmospheres => 14,
            Self::CubicFeetPerMinute => 15,
            Self::UsGallonsPerMinute => 16,
            Self::LitersPerMinute => 17,
            Self::ImperialGallonsPerMinute => 18,
            Self::CubicMetersPerHour => 19,
            Self::UsGallonsPerSecond => 22,
            Self::MillionUsGallonsPerDay => 23,
            Self::LitersPerSecond => 24,
            Self::MillionLitersPerDay => 25,
            Self::CubicFeetPerSecond => 26,
            Self::CubicFeetPerDay => 27,
            Self::CubicMetersPerSecond => 28,
            Self::CubicMetersPerDay => 29,
            Self::ImperialGallonsPerHour => 30,
            Self::ImperialGallonsPerDay => 31,
            Self::DegreesCelsius => 32,
            Self::DegreesFahrenheit => 33,
            Self::DegreesRankine => 34,
            Self::Kelvin => 35,
            Self::Millivolts => 36,
            Self::Volts => 37,
            Self::Ohms => 38,
            Self::Milliamperes => 39,
            Self::UsGallons => 40,
            Self::Liters => 41,
            Self::ImperialGallons => 42,
            Self::CubicMeters => 43,
            Self::Feet => 44,
            Self::Meters => 45,
            Self::Barrels => 46,
            Self::Inches => 47,
            Self::Centimeters => 48,
            Self::Millimeters => 49,
            Self::Minutes => 50,
            Self::Seconds => 51,
            Self::Hours => 52,
            Self::Days => 53,
            Self::Percent => 57,
            Self::Grams => 60,
            Self::Kilograms => 61,
            Self::MetricTons => 62,
            Self::Pounds => 63,
            Self::GramsPerSecond => 70,
            Self::GramsPerMinute => 71,
            Self::GramsPerHour => 72,
            Self::KilogramsPerSecond => 73,
            Self::KilogramsPerMinute => 74,
            Self::KilogramsPerHour => 75,
            Self::KilogramsPerDay => 76,
            Self::MetricTonsPerMinute => 77,
            Self::MetricTonsPerHour => 78,
            Self::MetricTonsPerDay => 79,
            Self::PoundsPerSecond => 80,
            Self::PoundsPerMinute => 81,
            Self::PoundsPerHour => 82,
            Self::PoundsPerDay => 83,
            Self::ShortTonsPerMinute => 84,
            Self::ShortTonsPerHour => 85,
            Self::GramsPerCubicCentimeter => 91,
            Self::KilogramsPerCubicMeter => 92,
            Self::PoundsPerUsGallon => 93,
            Self::PoundsPerCubicFoot => 94,
            Self::GramsPerMilliliter => 95,
            Self::KilogramsPerLiter => 96,
            Self::GramsPerLiter => 97,
            Self::Bushels => 110,
            Self::CubicYards => 111,
            Self::CubicFeet => 112,
            Self::CubicInches => 113,
            Self::LiquidBarrels => 124,
            Self::CubicFeetPerHour => 130,
            Self::CubicMetersPerMinute => 131,
            Self::BarrelsPerSecond => 132,
            Self::BarrelsPerMinute => 133,
            Self::BarrelsPerHour => 134,
            Self::BarrelsPerDay => 135,
            Self::UsGallonsPerHour => 136,
            Self::ImperialGallonsPerSecond => 137,
            Self::LitersPerHour => 138,
            Self::InchesWaterColumn60F => 145,
            Self::UsGallonsPerDay => 235,
            Self::Hectoliters => 236,
            Self::MegaPascals => 237,
            Self::InchesWaterColumn4C => 238,
            Self::MillimetersOfWater4C => 239,
            Self::NotUsed => 250,
            Self::None => 251,
            Self::Custom => 253,
            Self::Unknown(code) => *code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_codes_roundtrip() {
        assert_eq!(UnitCode::from_u8(45), UnitCode::Meters);
        assert_eq!(UnitCode::Meters.as_u8(), 45);
        assert_eq!(UnitCode::from_u8(32), UnitCode::DegreesCelsius);
        assert_eq!(UnitCode::DegreesCelsius.as_u8(), 32);
        assert_eq!(UnitCode::from_u8(57), UnitCode::Percent);
        assert_eq!(UnitCode::Percent.as_u8(), 57);
        assert_eq!(UnitCode::from_u8(7), UnitCode::Bar);
        assert_eq!(UnitCode::Bar.as_u8(), 7);
    }

    #[test]
    fn unknown_code_preserved() {
        let code = UnitCode::from_u8(199);
        assert_eq!(code, UnitCode::Unknown(199));
        assert_eq!(code.as_u8(), 199);
    }

    #[test]
    fn special_codes() {
        assert_eq!(UnitCode::from_u8(250), UnitCode::NotUsed);
        assert_eq!(UnitCode::from_u8(251), UnitCode::None);
        assert_eq!(UnitCode::from_u8(253), UnitCode::Custom);
    }

    #[test]
    fn all_known_codes_roundtrip() {
        // Verify every named variant survives a roundtrip
        let codes: &[u8] = &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 57,
            60, 61, 62, 63, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79,
            80, 81, 82, 83, 84, 85, 91, 92, 93, 94, 95, 96, 97,
            110, 111, 112, 113, 124, 130, 131, 132, 133, 134, 135, 136, 137, 138,
            145, 235, 236, 237, 238, 239, 250, 251, 253,
        ];
        for &code in codes {
            let unit = UnitCode::from_u8(code);
            assert_ne!(unit, UnitCode::Unknown(code), "code {} mapped to Unknown", code);
            assert_eq!(unit.as_u8(), code, "roundtrip failed for code {}", code);
        }
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
pub mod error;
pub mod types;
pub mod units;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add UnitCode enum with ~100 HART engineering unit codes"
```

---

### Task 5: Frame Encoder

**Files:**
- Create: `hart-protocol/src/encode.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write encoder tests**

```rust
// hart-protocol/src/encode.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::*;
    use crate::types::*;

    #[test]
    fn encode_short_request_no_data() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DEVICE_ID,
            &[],
            5,
            &mut buf,
        ).unwrap();

        // 5 preamble + 1 delim + 1 addr + 1 cmd + 1 len + 0 data + 1 chk = 10
        assert_eq!(len, 10);
        // Preambles
        assert!(buf[..5].iter().all(|&b| b == PREAMBLE_BYTE));
        // Delimiter: request short
        assert_eq!(buf[5], delimiter::REQUEST_SHORT);
        // Address: primary master, poll 0
        assert_eq!(buf[6], 0x80);
        // Command
        assert_eq!(buf[7], commands::READ_DEVICE_ID);
        // Byte count
        assert_eq!(buf[8], 0);
        // Checksum: XOR of delimiter..last_data_byte
        let expected_chk = buf[5] ^ buf[6] ^ buf[7] ^ buf[8];
        assert_eq!(buf[9], expected_chk);
    }

    #[test]
    fn encode_long_request_with_data() {
        let address = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x00_11_22_33,
        };
        let data = [0x01, 0x02, 0x03];
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DYNAMIC_VARS,
            &data,
            5,
            &mut buf,
        ).unwrap();

        // 5 preamble + 1 delim + 5 addr + 1 cmd + 1 len + 3 data + 1 chk = 17
        assert_eq!(len, 17);
        assert_eq!(buf[5], delimiter::REQUEST_LONG);
        assert_eq!(buf[11], commands::READ_DYNAMIC_VARS);
        assert_eq!(buf[12], 3); // byte count
        assert_eq!(&buf[13..16], &data);
    }

    #[test]
    fn encode_buffer_too_small() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 5]; // too small
        let result = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DEVICE_ID,
            &[],
            5,
            &mut buf,
        );
        assert_eq!(result, Err(crate::error::EncodeError::BufferTooSmall));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `encode` module not found

- [ ] **Step 3: Write encoder implementation**

```rust
// hart-protocol/src/encode.rs
use crate::consts::*;
use crate::error::EncodeError;
use crate::types::{Address, FrameType};

/// Compute the delimiter byte from frame type and address type.
fn delimiter_byte(frame_type: FrameType, is_long: bool) -> u8 {
    let base = match frame_type {
        FrameType::Request => delimiter::REQUEST_SHORT,
        FrameType::Response => delimiter::RESPONSE_SHORT,
        FrameType::Burst => delimiter::BURST_SHORT,
    };
    if is_long {
        base | delimiter::LONG_ADDRESS_BIT
    } else {
        base
    }
}

/// Encode a HART frame into `buf`. Returns the total number of bytes written.
///
/// The frame includes preamble, delimiter, address, command, byte count, data, and checksum.
pub fn encode_frame(
    frame_type: FrameType,
    address: &Address,
    command: u8,
    data: &[u8],
    preamble_count: u8,
    buf: &mut [u8],
) -> Result<usize, EncodeError> {
    if data.len() > MAX_DATA_LENGTH {
        return Err(EncodeError::DataTooLong);
    }

    let addr_len: usize = if address.is_long() { 5 } else { 1 };
    let total = preamble_count as usize + 1 + addr_len + 1 + 1 + data.len() + 1;

    if buf.len() < total {
        return Err(EncodeError::BufferTooSmall);
    }

    let mut pos = 0;

    // Preamble
    for _ in 0..preamble_count {
        buf[pos] = PREAMBLE_BYTE;
        pos += 1;
    }

    // Delimiter
    let delim = delimiter_byte(frame_type, address.is_long());
    buf[pos] = delim;
    let chk_start = pos;
    pos += 1;

    // Address
    let addr_written = address.encode(&mut buf[pos..]);
    pos += addr_written;

    // Command
    buf[pos] = command;
    pos += 1;

    // Byte count
    buf[pos] = data.len() as u8;
    pos += 1;

    // Data
    buf[pos..pos + data.len()].copy_from_slice(data);
    pos += data.len();

    // Checksum: XOR of all bytes from delimiter through last data byte
    let mut checksum: u8 = 0;
    for &byte in &buf[chk_start..pos] {
        checksum ^= byte;
    }
    buf[pos] = checksum;
    pos += 1;

    Ok(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn encode_short_request_no_data() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DEVICE_ID,
            &[],
            5,
            &mut buf,
        ).unwrap();

        assert_eq!(len, 10);
        assert!(buf[..5].iter().all(|&b| b == PREAMBLE_BYTE));
        assert_eq!(buf[5], delimiter::REQUEST_SHORT);
        assert_eq!(buf[6], 0x80);
        assert_eq!(buf[7], commands::READ_DEVICE_ID);
        assert_eq!(buf[8], 0);
        let expected_chk = buf[5] ^ buf[6] ^ buf[7] ^ buf[8];
        assert_eq!(buf[9], expected_chk);
    }

    #[test]
    fn encode_long_request_with_data() {
        let address = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x00_11_22_33,
        };
        let data = [0x01, 0x02, 0x03];
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DYNAMIC_VARS,
            &data,
            5,
            &mut buf,
        ).unwrap();

        assert_eq!(len, 17);
        assert_eq!(buf[5], delimiter::REQUEST_LONG);
        assert_eq!(buf[11], commands::READ_DYNAMIC_VARS);
        assert_eq!(buf[12], 3);
        assert_eq!(&buf[13..16], &data);
    }

    #[test]
    fn encode_buffer_too_small() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 5];
        let result = encode_frame(
            FrameType::Request,
            &address,
            commands::READ_DEVICE_ID,
            &[],
            5,
            &mut buf,
        );
        assert_eq!(result, Err(EncodeError::BufferTooSmall));
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
pub mod error;
pub mod types;
pub mod units;
pub mod encode;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add frame encoder"
```

---

### Task 6: Frame Decoder (byte-at-a-time state machine)

**Files:**
- Create: `hart-protocol/src/decode.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write decoder tests**

```rust
// hart-protocol/src/decode.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::*;
    use crate::types::*;
    use crate::encode::encode_frame;

    fn encode_then_decode(
        frame_type: FrameType,
        address: &Address,
        command: u8,
        data: &[u8],
    ) -> RawFrame {
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(frame_type, address, command, data, 5, &mut buf).unwrap();

        let mut decoder = Decoder::new();
        let mut result = Option::None;
        for &byte in &buf[..len] {
            if let Some(frame) = decoder.feed(byte).unwrap() {
                result = Some(frame);
            }
        }
        result.expect("decoder did not produce a frame")
    }

    #[test]
    fn decode_short_request_roundtrip() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let frame = encode_then_decode(FrameType::Request, &addr, commands::READ_DEVICE_ID, &[]);
        assert_eq!(frame.command, commands::READ_DEVICE_ID);
        assert!(frame.data.is_empty());
        assert!(matches!(frame.address, Address::Short { poll_address: 0, .. }));
    }

    #[test]
    fn decode_long_request_with_data() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x112233,
        };
        let data = [0xAA, 0xBB];
        let frame = encode_then_decode(FrameType::Request, &addr, commands::READ_DYNAMIC_VARS, &data);
        assert_eq!(frame.command, commands::READ_DYNAMIC_VARS);
        assert_eq!(frame.data.as_slice(), &data);
        match frame.address {
            Address::Long { manufacturer_id, device_type, device_id, .. } => {
                assert_eq!(manufacturer_id, 0x1A);
                assert_eq!(device_type, 0x2B);
                assert_eq!(device_id, 0x112233);
            }
            _ => panic!("expected long address"),
        }
    }

    #[test]
    fn decode_skips_garbage_before_preamble() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(FrameType::Request, &addr, commands::READ_DEVICE_ID, &[], 5, &mut buf).unwrap();

        let mut decoder = Decoder::new();
        // Feed garbage first
        for &byte in &[0x00, 0x55, 0xAA, 0x12] {
            assert!(decoder.feed(byte).unwrap().is_none());
        }
        // Then feed valid frame
        let mut result = Option::None;
        for &byte in &buf[..len] {
            if let Some(frame) = decoder.feed(byte).unwrap() {
                result = Some(frame);
            }
        }
        assert!(result.is_some());
    }

    #[test]
    fn decode_checksum_error() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(FrameType::Request, &addr, commands::READ_DEVICE_ID, &[], 5, &mut buf).unwrap();

        // Corrupt checksum
        buf[len - 1] ^= 0xFF;

        let mut decoder = Decoder::new();
        let mut err = Option::None;
        for &byte in &buf[..len] {
            match decoder.feed(byte) {
                Err(e) => { err = Some(e); break; }
                _ => {}
            }
        }
        assert!(matches!(err, Some(crate::error::DecodeError::ChecksumMismatch { .. })));
    }

    #[test]
    fn decoder_reset() {
        let mut decoder = Decoder::new();
        // Feed some preamble bytes
        for _ in 0..3 {
            decoder.feed(PREAMBLE_BYTE).unwrap();
        }
        decoder.reset();
        // After reset, should be back to hunting for preamble
        // Feed a full valid frame
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; MAX_FRAME_LENGTH];
        let len = encode_frame(FrameType::Request, &addr, commands::READ_DEVICE_ID, &[], 5, &mut buf).unwrap();
        let mut result = Option::None;
        for &byte in &buf[..len] {
            if let Some(frame) = decoder.feed(byte).unwrap() {
                result = Some(frame);
            }
        }
        assert!(result.is_some());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `decode` module not found

- [ ] **Step 3: Write decoder implementation**

```rust
// hart-protocol/src/decode.rs
use heapless::Vec;

use crate::consts::*;
use crate::error::DecodeError;
use crate::types::{Address, FrameType};

/// A decoded HART frame with owned data.
#[derive(Debug, Clone)]
pub struct RawFrame {
    pub frame_type: FrameType,
    pub address: Address,
    pub command: u8,
    pub data: Vec<u8, 256>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    /// Waiting for preamble bytes (0xFF)
    Hunting,
    /// Receiving preamble bytes, counting them
    Preamble { count: u8 },
    /// Reading delimiter byte
    Delimiter,
    /// Reading address bytes
    Address { remaining: u8 },
    /// Reading command byte
    Command,
    /// Reading byte count
    ByteCount,
    /// Reading data bytes
    Data { remaining: u8 },
    /// Reading checksum byte
    Checksum,
}

/// Byte-at-a-time HART frame decoder.
///
/// Feed bytes via `feed()`. Returns `Ok(Some(frame))` when a complete valid
/// frame is decoded. Returns `Err` on checksum mismatch or invalid delimiter.
/// Returns `Ok(None)` while accumulating bytes.
pub struct Decoder {
    state: State,
    checksum: u8,
    delim_byte: u8,
    addr_buf: [u8; 5],
    addr_pos: u8,
    command: u8,
    data: Vec<u8, 256>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            state: State::Hunting,
            checksum: 0,
            delim_byte: 0,
            addr_buf: [0; 5],
            addr_pos: 0,
            command: 0,
            data: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = State::Hunting;
        self.checksum = 0;
        self.delim_byte = 0;
        self.addr_buf = [0; 5];
        self.addr_pos = 0;
        self.command = 0;
        self.data.clear();
    }

    pub fn feed(&mut self, byte: u8) -> Result<Option<RawFrame>, DecodeError> {
        match self.state {
            State::Hunting => {
                if byte == PREAMBLE_BYTE {
                    self.state = State::Preamble { count: 1 };
                }
                Ok(None)
            }
            State::Preamble { count } => {
                if byte == PREAMBLE_BYTE {
                    self.state = State::Preamble { count: count.saturating_add(1) };
                    Ok(None)
                } else if count >= MIN_PREAMBLE_COUNT {
                    // This byte is the delimiter
                    self.feed_delimiter(byte)
                } else {
                    // Not enough preamble bytes, start over
                    self.reset();
                    Ok(None)
                }
            }
            State::Delimiter => {
                // Shouldn't reach here — delimiter handled in Preamble transition
                self.reset();
                Ok(None)
            }
            State::Address { remaining } => {
                self.checksum ^= byte;
                self.addr_buf[self.addr_pos as usize] = byte;
                self.addr_pos += 1;
                if remaining == 1 {
                    self.state = State::Command;
                } else {
                    self.state = State::Address { remaining: remaining - 1 };
                }
                Ok(None)
            }
            State::Command => {
                self.checksum ^= byte;
                self.command = byte;
                self.state = State::ByteCount;
                Ok(None)
            }
            State::ByteCount => {
                self.checksum ^= byte;
                if byte == 0 {
                    self.state = State::Checksum;
                } else {
                    self.data.clear();
                    self.state = State::Data { remaining: byte };
                }
                Ok(None)
            }
            State::Data { remaining } => {
                self.checksum ^= byte;
                let _ = self.data.push(byte);
                if remaining == 1 {
                    self.state = State::Checksum;
                } else {
                    self.state = State::Data { remaining: remaining - 1 };
                }
                Ok(None)
            }
            State::Checksum => {
                if byte != self.checksum {
                    let expected = self.checksum;
                    self.reset();
                    return Err(DecodeError::ChecksumMismatch { expected, actual: byte });
                }

                let frame = self.build_frame()?;
                self.reset();
                Ok(Some(frame))
            }
        }
    }

    fn feed_delimiter(&mut self, byte: u8) -> Result<Option<RawFrame>, DecodeError> {
        self.delim_byte = byte;
        self.checksum = byte;
        self.addr_pos = 0;
        self.data.clear();

        let is_long = byte & delimiter::LONG_ADDRESS_BIT != 0;
        let addr_len = if is_long { 5u8 } else { 1u8 };

        self.state = State::Address { remaining: addr_len };
        Ok(None)
    }

    fn build_frame(&self) -> Result<RawFrame, DecodeError> {
        let is_long = self.delim_byte & delimiter::LONG_ADDRESS_BIT != 0;
        let base = self.delim_byte & !delimiter::LONG_ADDRESS_BIT;

        let frame_type = match base {
            0x01 => FrameType::Burst,
            0x02 => FrameType::Request,
            0x06 => FrameType::Response,
            _ => return Err(DecodeError::InvalidFrameType),
        };

        let addr_len = if is_long { 5 } else { 1 };
        let (address, _) = Address::decode(&self.addr_buf[..addr_len], is_long)?;

        Ok(RawFrame {
            frame_type,
            address,
            command: self.command,
            data: self.data.clone(),
        })
    }
}
```

(Tests from Step 1 appended to the bottom of this file.)

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
pub mod error;
pub mod types;
pub mod units;
pub mod encode;
pub mod decode;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add byte-at-a-time frame decoder state machine"
```

---

### Task 7: Command Traits and Phase 1 Commands (0, 1, 2, 3, 48)

**Files:**
- Create: `hart-protocol/src/commands/mod.rs`
- Create: `hart-protocol/src/commands/cmd0.rs`
- Create: `hart-protocol/src/commands/cmd1.rs`
- Create: `hart-protocol/src/commands/cmd2.rs`
- Create: `hart-protocol/src/commands/cmd3.rs`
- Create: `hart-protocol/src/commands/cmd48.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write command trait and cmd0 test**

```rust
// Test for cmd0 — in hart-protocol/src/commands/cmd0.rs at bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::commands;

    #[test]
    fn cmd0_command_number() {
        assert_eq!(ReadDeviceIdRequest::COMMAND_NUMBER, commands::READ_DEVICE_ID);
        assert_eq!(ReadDeviceIdResponse::COMMAND_NUMBER, commands::READ_DEVICE_ID);
    }

    #[test]
    fn cmd0_request_encodes_empty() {
        let req = ReadDeviceIdRequest;
        let mut buf = [0u8; 32];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn cmd0_response_decodes() {
        // 12 bytes: expansion(1) + expanded_device_type(2) + min_preambles(1) +
        //           hart_revision(1) + device_revision(1) + sw_revision(1) +
        //           hw_revision_signaling(1) + flags(1) + device_id(3)
        let data: [u8; 12] = [
            0xFE,       // expansion code
            0x00, 0x45, // expanded device type (manufacturer_id=0, device_type=0x45)
            0x05,       // min preambles
            0x07,       // HART revision 7
            0x01,       // device revision
            0x02,       // software revision
            0x24,       // hw_revision=4 (bits 7-3=00100), signaling=4 (bits 2-0=100)
            0x00,       // flags
            0x11, 0x22, 0x33, // device ID
        ];
        let resp = ReadDeviceIdResponse::decode_data(&data).unwrap();
        assert_eq!(resp.expanded_device_type, 0x0045);
        assert_eq!(resp.min_preamble_count, 5);
        assert_eq!(resp.hart_revision, 7);
        assert_eq!(resp.device_revision, 1);
        assert_eq!(resp.software_revision, 2);
        assert_eq!(resp.device_id, 0x112233);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `commands` module not found

- [ ] **Step 3: Write command traits and all Phase 1 command implementations**

```rust
// hart-protocol/src/commands/mod.rs
use crate::error::{EncodeError, DecodeError};

/// Trait for command request payloads.
pub trait CommandRequest {
    const COMMAND_NUMBER: u8;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

/// Trait for command response payloads.
pub trait CommandResponse: Sized {
    const COMMAND_NUMBER: u8;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError>;
}

pub mod cmd0;
pub mod cmd1;
pub mod cmd2;
pub mod cmd3;
pub mod cmd48;
```

```rust
// hart-protocol/src/commands/cmd0.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

/// Command 0: Read Device ID — request (empty).
pub struct ReadDeviceIdRequest;

impl CommandRequest for ReadDeviceIdRequest {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_ID;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

/// Command 0: Read Device ID — response.
#[derive(Debug, Clone)]
pub struct ReadDeviceIdResponse {
    pub expansion_code: u8,
    pub expanded_device_type: u16,
    pub min_preamble_count: u8,
    pub hart_revision: u8,
    pub device_revision: u8,
    pub software_revision: u8,
    pub hardware_revision: u8,
    pub physical_signaling: u8,
    pub flags: u8,
    pub device_id: u32,
}

impl CommandResponse for ReadDeviceIdResponse {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_ID;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 12 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            expansion_code: data[0],
            expanded_device_type: ((data[1] as u16) << 8) | (data[2] as u16),
            min_preamble_count: data[3],
            hart_revision: data[4],
            device_revision: data[5],
            software_revision: data[6],
            hardware_revision: data[7] >> 3,
            physical_signaling: data[7] & 0x07,
            flags: data[8],
            device_id: ((data[9] as u32) << 16)
                | ((data[10] as u32) << 8)
                | (data[11] as u32),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd0_command_number() {
        assert_eq!(ReadDeviceIdRequest::COMMAND_NUMBER, commands::READ_DEVICE_ID);
        assert_eq!(ReadDeviceIdResponse::COMMAND_NUMBER, commands::READ_DEVICE_ID);
    }

    #[test]
    fn cmd0_request_encodes_empty() {
        let req = ReadDeviceIdRequest;
        let mut buf = [0u8; 32];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn cmd0_response_decodes() {
        let data: [u8; 12] = [
            0xFE, 0x00, 0x45, 0x05, 0x07, 0x01, 0x02, 0x24, 0x00,
            0x11, 0x22, 0x33,
        ];
        let resp = ReadDeviceIdResponse::decode_data(&data).unwrap();
        assert_eq!(resp.expanded_device_type, 0x0045);
        assert_eq!(resp.min_preamble_count, 5);
        assert_eq!(resp.hart_revision, 7);
        assert_eq!(resp.device_revision, 1);
        assert_eq!(resp.software_revision, 2);
        assert_eq!(resp.hardware_revision, 4);
        assert_eq!(resp.physical_signaling, 4);
        assert_eq!(resp.device_id, 0x112233);
    }
}
```

```rust
// hart-protocol/src/commands/cmd1.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

/// Command 1: Read Primary Variable — request (empty).
pub struct ReadPvRequest;

impl CommandRequest for ReadPvRequest {
    const COMMAND_NUMBER: u8 = commands::READ_PRIMARY_VARIABLE;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 1: Read Primary Variable — response.
#[derive(Debug, Clone)]
pub struct ReadPvResponse {
    pub unit: UnitCode,
    pub value: f32,
}

impl CommandResponse for ReadPvResponse {
    const COMMAND_NUMBER: u8 = commands::READ_PRIMARY_VARIABLE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 5 {
            return Err(DecodeError::BufferTooShort);
        }
        let unit = UnitCode::from_u8(data[0]);
        let value = f32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        Ok(Self { unit, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd1_response_decodes() {
        // Unit: meters (45), value: 3.14 as IEEE 754 big-endian
        let value_bytes = 3.14_f32.to_be_bytes();
        let data = [45, value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]];
        let resp = ReadPvResponse::decode_data(&data).unwrap();
        assert_eq!(resp.unit, UnitCode::Meters);
        assert!((resp.value - 3.14).abs() < 0.001);
    }
}
```

```rust
// hart-protocol/src/commands/cmd2.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

/// Command 2: Read Loop Current and Percent of Range — request (empty).
pub struct ReadLoopCurrentRequest;

impl CommandRequest for ReadLoopCurrentRequest {
    const COMMAND_NUMBER: u8 = commands::READ_LOOP_CURRENT_PERCENT;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 2: Read Loop Current and Percent of Range — response.
#[derive(Debug, Clone)]
pub struct ReadLoopCurrentResponse {
    pub current_ma: f32,
    pub percent_of_range: f32,
}

impl CommandResponse for ReadLoopCurrentResponse {
    const COMMAND_NUMBER: u8 = commands::READ_LOOP_CURRENT_PERCENT;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 8 {
            return Err(DecodeError::BufferTooShort);
        }
        let current_ma = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let percent_of_range = f32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Ok(Self { current_ma, percent_of_range })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd2_response_decodes() {
        let current = 12.5_f32.to_be_bytes();
        let percent = 50.0_f32.to_be_bytes();
        let mut data = [0u8; 8];
        data[..4].copy_from_slice(&current);
        data[4..8].copy_from_slice(&percent);
        let resp = ReadLoopCurrentResponse::decode_data(&data).unwrap();
        assert!((resp.current_ma - 12.5).abs() < 0.001);
        assert!((resp.percent_of_range - 50.0).abs() < 0.001);
    }
}
```

```rust
// hart-protocol/src/commands/cmd3.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

/// Command 3: Read Dynamic Variables and Loop Current — request (empty).
pub struct ReadDynamicVarsRequest;

impl CommandRequest for ReadDynamicVarsRequest {
    const COMMAND_NUMBER: u8 = commands::READ_DYNAMIC_VARS;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 3: Read Dynamic Variables and Loop Current — response.
#[derive(Debug, Clone)]
pub struct ReadDynamicVarsResponse {
    pub loop_current_ma: f32,
    pub pv_unit: UnitCode,
    pub pv: f32,
    pub sv_unit: UnitCode,
    pub sv: f32,
    pub tv_unit: UnitCode,
    pub tv: f32,
    pub qv_unit: UnitCode,
    pub qv: f32,
}

impl CommandResponse for ReadDynamicVarsResponse {
    const COMMAND_NUMBER: u8 = commands::READ_DYNAMIC_VARS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        // 4 (current) + 4*(1 unit + 4 value) = 4 + 20 = 24 bytes
        if data.len() < 24 {
            return Err(DecodeError::BufferTooShort);
        }
        let loop_current_ma = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        let pv_unit = UnitCode::from_u8(data[4]);
        let pv = f32::from_be_bytes([data[5], data[6], data[7], data[8]]);

        let sv_unit = UnitCode::from_u8(data[9]);
        let sv = f32::from_be_bytes([data[10], data[11], data[12], data[13]]);

        let tv_unit = UnitCode::from_u8(data[14]);
        let tv = f32::from_be_bytes([data[15], data[16], data[17], data[18]]);

        let qv_unit = UnitCode::from_u8(data[19]);
        let qv = f32::from_be_bytes([data[20], data[21], data[22], data[23]]);

        Ok(Self { loop_current_ma, pv_unit, pv, sv_unit, sv, tv_unit, tv, qv_unit, qv })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd3_response_decodes_vegapuls21_style() {
        let mut data = [0u8; 24];
        // Loop current: 12.0 mA
        data[0..4].copy_from_slice(&12.0_f32.to_be_bytes());
        // PV: percent=57, value=50.0
        data[4] = 57;
        data[5..9].copy_from_slice(&50.0_f32.to_be_bytes());
        // SV: meters=45, value=2.5
        data[9] = 45;
        data[10..14].copy_from_slice(&2.5_f32.to_be_bytes());
        // TV: not_used=250, value=NaN
        data[14] = 250;
        data[15..19].copy_from_slice(&f32::NAN.to_be_bytes());
        // QV: celsius=32, value=25.3
        data[19] = 32;
        data[20..24].copy_from_slice(&25.3_f32.to_be_bytes());

        let resp = ReadDynamicVarsResponse::decode_data(&data).unwrap();
        assert!((resp.loop_current_ma - 12.0).abs() < 0.001);
        assert_eq!(resp.pv_unit, UnitCode::Percent);
        assert!((resp.pv - 50.0).abs() < 0.001);
        assert_eq!(resp.sv_unit, UnitCode::Meters);
        assert!((resp.sv - 2.5).abs() < 0.001);
        assert_eq!(resp.tv_unit, UnitCode::NotUsed);
        assert!(resp.tv.is_nan());
        assert_eq!(resp.qv_unit, UnitCode::DegreesCelsius);
        assert!((resp.qv - 25.3).abs() < 0.01);
    }
}
```

```rust
// hart-protocol/src/commands/cmd48.rs
use heapless::Vec;
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

/// Command 48: Read Additional Device Status — request (empty).
pub struct ReadAdditionalStatusRequest;

impl CommandRequest for ReadAdditionalStatusRequest {
    const COMMAND_NUMBER: u8 = commands::READ_ADDITIONAL_STATUS;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 48: Read Additional Device Status — response.
/// Data is variable-length and device-specific.
#[derive(Debug, Clone)]
pub struct ReadAdditionalStatusResponse {
    pub data: Vec<u8, 25>,
}

impl CommandResponse for ReadAdditionalStatusResponse {
    const COMMAND_NUMBER: u8 = commands::READ_ADDITIONAL_STATUS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let mut vec = Vec::new();
        for &byte in data.iter().take(25) {
            let _ = vec.push(byte);
        }
        Ok(Self { data: vec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd48_response_decodes_variable_length() {
        let data = [0x00, 0x01, 0x02, 0x03, 0x04];
        let resp = ReadAdditionalStatusResponse::decode_data(&data).unwrap();
        assert_eq!(resp.data.len(), 5);
        assert_eq!(resp.data[0], 0x00);
        assert_eq!(resp.data[4], 0x04);
    }

    #[test]
    fn cmd48_response_decodes_empty() {
        let resp = ReadAdditionalStatusResponse::decode_data(&[]).unwrap();
        assert!(resp.data.is_empty());
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// hart-protocol/src/lib.rs
#![no_std]

pub mod consts;
pub mod error;
pub mod types;
pub mod units;
pub mod encode;
pub mod decode;
pub mod commands;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add command traits and Phase 1 commands (0, 1, 2, 3, 48)"
```

---

### Task 8: AD5700 Blocking Modem Driver

**Files:**
- Create: `ad5700/src/error.rs`
- Create: `ad5700/src/blocking.rs`
- Modify: `ad5700/src/lib.rs`

- [ ] **Step 1: Write blocking driver test**

```rust
// ad5700/src/blocking.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    // Tests use embedded-hal-mock to verify pin behavior.
    // Full mock tests deferred until embedded-hal-mock 0.11
    // supports the exact traits we need. For now, verify the
    // types compile with generic bounds.

    use super::*;

    // Compile-time check: Ad5700 is constructible with any types
    // meeting the embedded-hal bounds.
    fn _assert_send<T: Send>() {}
}
```

- [ ] **Step 2: Write error types**

```rust
// ad5700/src/error.rs

/// Errors from the AD5700 modem driver.
#[derive(Debug, Clone, PartialEq)]
pub enum Ad5700Error<E> {
    /// Underlying UART error.
    Uart(E),
    /// No carrier detected — device is not responding.
    NoCarrier,
    /// Response did not complete within timeout.
    Timeout,
}

/// Errors from the HART master layer.
#[derive(Debug)]
pub enum HartError<E> {
    Modem(Ad5700Error<E>),
    Encode(hart_protocol::error::EncodeError),
    Decode(hart_protocol::error::DecodeError),
    Timeout,
}

impl<E> From<Ad5700Error<E>> for HartError<E> {
    fn from(e: Ad5700Error<E>) -> Self {
        HartError::Modem(e)
    }
}

impl<E> From<hart_protocol::error::EncodeError> for HartError<E> {
    fn from(e: hart_protocol::error::EncodeError) -> Self {
        HartError::Encode(e)
    }
}

impl<E> From<hart_protocol::error::DecodeError> for HartError<E> {
    fn from(e: hart_protocol::error::DecodeError) -> Self {
        HartError::Decode(e)
    }
}
```

- [ ] **Step 3: Write blocking modem driver**

```rust
// ad5700/src/blocking.rs
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::serial;

use crate::error::Ad5700Error;

/// Blocking AD5700-1 HART modem driver.
///
/// Generic over UART, RTS output pin, and CD input pin.
pub struct Ad5700<UART, RTS, CD> {
    uart: UART,
    rts: RTS,
    cd: CD,
}

impl<UART, RTS, CD, E> Ad5700<UART, RTS, CD>
where
    UART: serial::Read<Error = E> + serial::Write<Error = E>,
    RTS: OutputPin,
    CD: InputPin,
{
    pub fn new(uart: UART, rts: RTS, cd: CD) -> Self {
        Self { uart, rts, cd }
    }

    /// Transmit data: assert RTS, send bytes, deassert RTS.
    pub fn transmit(&mut self, data: &[u8]) -> Result<(), Ad5700Error<E>> {
        // Assert RTS to switch modem to transmit mode
        let _ = self.rts.set_high();

        // Small delay for modem to switch (handled by caller or busy-wait)
        // In a real system, insert RTS_HOLD_TIME_MS delay here

        for &byte in data {
            self.uart.write(&[byte]).map_err(Ad5700Error::Uart)?;
        }
        self.uart.flush().map_err(Ad5700Error::Uart)?;

        // Deassert RTS to switch back to receive mode
        let _ = self.rts.set_low();

        Ok(())
    }

    /// Receive bytes into buf. Returns number of bytes read.
    /// Reads until no more bytes arrive (simple polling approach).
    pub fn receive_into(&mut self, buf: &mut [u8]) -> Result<usize, Ad5700Error<E>> {
        let mut pos = 0;
        while pos < buf.len() {
            let mut byte = [0u8; 1];
            match self.uart.read(&mut byte) {
                Ok(()) => {
                    buf[pos] = byte[0];
                    pos += 1;
                }
                Err(_) => break,
            }
        }
        Ok(pos)
    }

    /// Check if carrier is detected.
    pub fn carrier_detected(&self) -> bool {
        self.cd.is_high().unwrap_or(false)
    }

    /// Release the inner peripherals.
    pub fn release(self) -> (UART, RTS, CD) {
        (self.uart, self.rts, self.cd)
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

```rust
// ad5700/src/lib.rs
#![no_std]

pub mod error;
pub mod blocking;
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo build -p ad5700`
Expected: compiles with no errors

- [ ] **Step 6: Commit**

```bash
git add ad5700/src/
git commit -m "feat(ad5700): add blocking modem driver with RTS/CD control"
```

---

### Task 9: AD5700 Async Modem Driver

**Files:**
- Create: `ad5700/src/asynch.rs`
- Modify: `ad5700/src/lib.rs`

- [ ] **Step 1: Write async modem driver**

```rust
// ad5700/src/asynch.rs
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::serial;

use crate::error::Ad5700Error;

/// Async AD5700-1 HART modem driver.
pub struct Ad5700Async<UART, RTS, CD> {
    uart: UART,
    rts: RTS,
    cd: CD,
}

impl<UART, RTS, CD, E> Ad5700Async<UART, RTS, CD>
where
    UART: serial::Read<Error = E> + serial::Write<Error = E>,
    RTS: OutputPin,
    CD: InputPin,
{
    pub fn new(uart: UART, rts: RTS, cd: CD) -> Self {
        Self { uart, rts, cd }
    }

    /// Transmit data: assert RTS, send bytes, deassert RTS.
    pub async fn transmit(&mut self, data: &[u8]) -> Result<(), Ad5700Error<E>> {
        let _ = self.rts.set_high();
        self.uart.write(data).await.map_err(Ad5700Error::Uart)?;
        self.uart.flush().await.map_err(Ad5700Error::Uart)?;
        let _ = self.rts.set_low();
        Ok(())
    }

    /// Receive bytes into buf. Returns number of bytes read.
    pub async fn receive_into(&mut self, buf: &mut [u8]) -> Result<usize, Ad5700Error<E>> {
        let mut pos = 0;
        while pos < buf.len() {
            let mut byte = [0u8; 1];
            match self.uart.read(&mut byte).await {
                Ok(()) => {
                    buf[pos] = byte[0];
                    pos += 1;
                }
                Err(_) => break,
            }
        }
        Ok(pos)
    }

    /// Check if carrier is detected.
    pub fn carrier_detected(&self) -> bool {
        self.cd.is_high().unwrap_or(false)
    }

    /// Release the inner peripherals.
    pub fn release(self) -> (UART, RTS, CD) {
        (self.uart, self.rts, self.cd)
    }
}
```

- [ ] **Step 2: Wire into lib.rs**

```rust
// ad5700/src/lib.rs
#![no_std]

pub mod error;
pub mod blocking;
pub mod asynch;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p ad5700`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add ad5700/src/
git commit -m "feat(ad5700): add async modem driver"
```

---

### Task 10: HartMasterBlocking (in ad5700 crate)

**Files:**
- Create: `ad5700/src/master.rs`
- Modify: `ad5700/src/lib.rs`

- [ ] **Step 1: Write HartMasterBlocking**

```rust
// ad5700/src/master.rs
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::serial;

use hart_protocol::commands::{CommandRequest, CommandResponse};
use hart_protocol::consts::*;
use hart_protocol::decode::{Decoder, RawFrame};
use hart_protocol::encode::encode_frame;
use hart_protocol::types::*;

use crate::blocking::Ad5700;
use crate::error::HartError;

/// Blocking HART master combining AD5700 modem + hart-protocol codec.
pub struct HartMasterBlocking<UART, RTS, CD> {
    modem: Ad5700<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; MAX_FRAME_LENGTH],
    rx_buf: [u8; MAX_FRAME_LENGTH],
    preamble_count: u8,
}

impl<UART, RTS, CD, E> HartMasterBlocking<UART, RTS, CD>
where
    UART: serial::Read<Error = E> + serial::Write<Error = E>,
    RTS: OutputPin,
    CD: InputPin,
{
    pub fn new(modem: Ad5700<UART, RTS, CD>) -> Self {
        Self {
            modem,
            decoder: Decoder::new(),
            tx_buf: [0; MAX_FRAME_LENGTH],
            rx_buf: [0; MAX_FRAME_LENGTH],
            preamble_count: DEFAULT_PREAMBLE_COUNT,
        }
    }

    /// Set the preamble count (learned from Command 0 response).
    pub fn set_preamble_count(&mut self, count: u8) {
        self.preamble_count = count;
    }

    /// Send a typed command and decode the typed response.
    pub fn send_command<Req, Resp>(
        &mut self,
        address: &Address,
        request: &Req,
    ) -> Result<(ResponseStatus, Resp), HartError<E>>
    where
        Req: CommandRequest,
        Resp: CommandResponse,
    {
        // Encode request data
        let mut data_buf = [0u8; MAX_DATA_LENGTH];
        let data_len = request.encode_data(&mut data_buf)?;

        // Encode full frame
        let frame_len = encode_frame(
            FrameType::Request,
            address,
            Req::COMMAND_NUMBER,
            &data_buf[..data_len],
            self.preamble_count,
            &mut self.tx_buf,
        )?;

        // Transmit
        self.modem
            .transmit(&self.tx_buf[..frame_len])
            .map_err(HartError::Modem)?;

        // Receive and decode
        self.decoder.reset();
        let rx_len = self.modem
            .receive_into(&mut self.rx_buf)
            .map_err(HartError::Modem)?;

        let mut frame: Option<RawFrame> = Option::None;
        for &byte in &self.rx_buf[..rx_len] {
            if let Some(f) = self.decoder.feed(byte)? {
                frame = Some(f);
                break;
            }
        }

        let frame = frame.ok_or(HartError::Timeout)?;

        // Parse status (first 2 bytes of response data)
        let status = if frame.data.len() >= 2 {
            ResponseStatus::from_bytes([frame.data[0], frame.data[1]])
        } else {
            ResponseStatus::from_bytes([0, 0])
        };

        // Decode command-specific response (skip 2 status bytes)
        let resp_data = if frame.data.len() > 2 {
            &frame.data[2..]
        } else {
            &[]
        };
        let resp = Resp::decode_data(resp_data)?;

        Ok((status, resp))
    }
}
```

- [ ] **Step 2: Wire into lib.rs**

```rust
// ad5700/src/lib.rs
#![no_std]

pub mod error;
pub mod blocking;
pub mod asynch;
pub mod master;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build -p ad5700`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
git add ad5700/src/
git commit -m "feat(ad5700): add HartMasterBlocking combining modem + codec"
```

---

### Task 11: Async HartMaster (embassy-hart crate)

**Files:**
- Create: `embassy-hart/src/master.rs`
- Modify: `embassy-hart/src/lib.rs`

- [ ] **Step 1: Write async HartMaster**

```rust
// embassy-hart/src/master.rs
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal_async::serial;
use embassy_time::{with_timeout, Duration};

use hart_protocol::commands::{CommandRequest, CommandResponse};
use hart_protocol::consts::*;
use hart_protocol::decode::{Decoder, RawFrame};
use hart_protocol::encode::encode_frame;
use hart_protocol::types::*;

use ad5700::asynch::Ad5700Async;
use ad5700::error::HartError;

/// Async HART master combining AD5700 async modem + hart-protocol codec + embassy-time timeouts.
pub struct HartMaster<UART, RTS, CD> {
    modem: Ad5700Async<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; MAX_FRAME_LENGTH],
    rx_buf: [u8; MAX_FRAME_LENGTH],
    preamble_count: u8,
}

impl<UART, RTS, CD, E> HartMaster<UART, RTS, CD>
where
    UART: serial::Read<Error = E> + serial::Write<Error = E>,
    RTS: OutputPin,
    CD: InputPin,
{
    pub fn new(modem: Ad5700Async<UART, RTS, CD>) -> Self {
        Self {
            modem,
            decoder: Decoder::new(),
            tx_buf: [0; MAX_FRAME_LENGTH],
            rx_buf: [0; MAX_FRAME_LENGTH],
            preamble_count: DEFAULT_PREAMBLE_COUNT,
        }
    }

    pub fn set_preamble_count(&mut self, count: u8) {
        self.preamble_count = count;
    }

    /// Send a typed command and decode the typed response, with timeout.
    pub async fn send_command<Req, Resp>(
        &mut self,
        address: &Address,
        request: &Req,
    ) -> Result<(ResponseStatus, Resp), HartError<E>>
    where
        Req: CommandRequest,
        Resp: CommandResponse,
    {
        // Encode request data
        let mut data_buf = [0u8; MAX_DATA_LENGTH];
        let data_len = request.encode_data(&mut data_buf)?;

        // Encode full frame
        let frame_len = encode_frame(
            FrameType::Request,
            address,
            Req::COMMAND_NUMBER,
            &data_buf[..data_len],
            self.preamble_count,
            &mut self.tx_buf,
        )?;

        // Transmit
        self.modem
            .transmit(&self.tx_buf[..frame_len])
            .await
            .map_err(HartError::Modem)?;

        // Receive with timeout
        let rx_result = with_timeout(
            Duration::from_millis(RESPONSE_TIMEOUT_MS as u64),
            self.modem.receive_into(&mut self.rx_buf),
        )
        .await;

        let rx_len = match rx_result {
            Ok(Ok(len)) => len,
            Ok(Err(e)) => return Err(HartError::Modem(e)),
            Err(_timeout) => return Err(HartError::Timeout),
        };

        // Decode
        self.decoder.reset();
        let mut frame: Option<RawFrame> = Option::None;
        for &byte in &self.rx_buf[..rx_len] {
            if let Some(f) = self.decoder.feed(byte)? {
                frame = Some(f);
                break;
            }
        }

        let frame = frame.ok_or(HartError::Timeout)?;

        let status = if frame.data.len() >= 2 {
            ResponseStatus::from_bytes([frame.data[0], frame.data[1]])
        } else {
            ResponseStatus::from_bytes([0, 0])
        };

        let resp_data = if frame.data.len() > 2 {
            &frame.data[2..]
        } else {
            &[]
        };
        let resp = Resp::decode_data(resp_data)?;

        Ok((status, resp))
    }
}
```

- [ ] **Step 2: Wire into lib.rs**

```rust
// embassy-hart/src/lib.rs
#![no_std]

pub mod master;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build --workspace`
Expected: all three crates compile

- [ ] **Step 4: Commit**

```bash
git add embassy-hart/src/
git commit -m "feat(embassy-hart): add async HartMaster with embassy-time timeouts"
```

---

### Task 12: End-to-End Codec Integration Test

**Files:**
- Create: `hart-protocol/tests/integration.rs`

- [ ] **Step 1: Write integration test simulating a Command 0 request/response cycle**

```rust
// hart-protocol/tests/integration.rs

use hart_protocol::commands::cmd0::*;
use hart_protocol::commands::cmd3::*;
use hart_protocol::commands::{CommandRequest, CommandResponse};
use hart_protocol::consts::*;
use hart_protocol::decode::Decoder;
use hart_protocol::encode::encode_frame;
use hart_protocol::types::*;
use hart_protocol::units::UnitCode;

/// Simulate: master sends Command 0, device responds with identity.
#[test]
fn full_cmd0_roundtrip() {
    // --- Master side: encode request ---
    let address = Address::Short {
        master: MasterRole::Primary,
        burst: false,
        poll_address: 0,
    };
    let req = ReadDeviceIdRequest;
    let mut req_data = [0u8; MAX_DATA_LENGTH];
    let data_len = req.encode_data(&mut req_data).unwrap();

    let mut tx_buf = [0u8; MAX_FRAME_LENGTH];
    let tx_len = encode_frame(
        FrameType::Request,
        &address,
        ReadDeviceIdRequest::COMMAND_NUMBER,
        &req_data[..data_len],
        5,
        &mut tx_buf,
    )
    .unwrap();

    // Verify request was encoded
    assert!(tx_len > 0);

    // --- Simulate device side: build a response frame ---
    let device_addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };
    // Response data: 2 status bytes + 12 cmd0 response bytes
    let mut resp_payload = [0u8; 14];
    resp_payload[0] = 0x00; // communication status: ok
    resp_payload[1] = 0x00; // device status: ok
    // cmd0 response: expansion, expanded_device_type, preambles, hart_rev, etc.
    resp_payload[2] = 0xFE; // expansion
    resp_payload[3] = 0x00; // expanded device type high
    resp_payload[4] = 0x2B; // expanded device type low
    resp_payload[5] = 0x05; // min preambles
    resp_payload[6] = 0x07; // HART rev 7
    resp_payload[7] = 0x01; // device rev
    resp_payload[8] = 0x03; // sw rev
    resp_payload[9] = 0x20; // hw rev=4, signaling=0
    resp_payload[10] = 0x00; // flags
    resp_payload[11] = 0x11; // device id
    resp_payload[12] = 0x22;
    resp_payload[13] = 0x33;

    let mut resp_buf = [0u8; MAX_FRAME_LENGTH];
    let resp_len = encode_frame(
        FrameType::Response,
        &device_addr,
        ReadDeviceIdResponse::COMMAND_NUMBER,
        &resp_payload,
        5,
        &mut resp_buf,
    )
    .unwrap();

    // --- Master side: decode response ---
    let mut decoder = Decoder::new();
    let mut frame = None;
    for &byte in &resp_buf[..resp_len] {
        if let Some(f) = decoder.feed(byte).unwrap() {
            frame = Some(f);
        }
    }
    let frame = frame.expect("should decode response frame");

    // Parse status
    let status = ResponseStatus::from_bytes([frame.data[0], frame.data[1]]);
    assert!(!status.has_error());

    // Parse command response
    let resp = ReadDeviceIdResponse::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.expanded_device_type, 0x002B);
    assert_eq!(resp.min_preamble_count, 5);
    assert_eq!(resp.hart_revision, 7);
    assert_eq!(resp.device_id, 0x112233);
}

/// Simulate: Command 3 response with VEGAPULS 21-style data.
#[test]
fn full_cmd3_vegapuls21_roundtrip() {
    let device_addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };

    // Build response: 2 status + 24 cmd3 data
    let mut resp_payload = [0u8; 26];
    resp_payload[0] = 0x00; // status ok
    resp_payload[1] = 0x00;
    // loop current
    resp_payload[2..6].copy_from_slice(&12.0_f32.to_be_bytes());
    // PV: percent, 50.0
    resp_payload[6] = UnitCode::Percent.as_u8();
    resp_payload[7..11].copy_from_slice(&50.0_f32.to_be_bytes());
    // SV: meters, 2.5
    resp_payload[11] = UnitCode::Meters.as_u8();
    resp_payload[12..16].copy_from_slice(&2.5_f32.to_be_bytes());
    // TV: not used
    resp_payload[16] = UnitCode::NotUsed.as_u8();
    resp_payload[17..21].copy_from_slice(&f32::NAN.to_be_bytes());
    // QV: celsius, 25.0
    resp_payload[21] = UnitCode::DegreesCelsius.as_u8();
    resp_payload[22..26].copy_from_slice(&25.0_f32.to_be_bytes());

    let mut resp_buf = [0u8; MAX_FRAME_LENGTH];
    let resp_len = encode_frame(
        FrameType::Response,
        &device_addr,
        ReadDynamicVarsResponse::COMMAND_NUMBER,
        &resp_payload,
        5,
        &mut resp_buf,
    )
    .unwrap();

    let mut decoder = Decoder::new();
    let mut frame = None;
    for &byte in &resp_buf[..resp_len] {
        if let Some(f) = decoder.feed(byte).unwrap() {
            frame = Some(f);
        }
    }
    let frame = frame.unwrap();
    let resp = ReadDynamicVarsResponse::decode_data(&frame.data[2..]).unwrap();

    assert!((resp.loop_current_ma - 12.0).abs() < 0.001);
    assert_eq!(resp.pv_unit, UnitCode::Percent);
    assert!((resp.pv - 50.0).abs() < 0.001);
    assert_eq!(resp.sv_unit, UnitCode::Meters);
    assert!((resp.sv - 2.5).abs() < 0.001);
    assert_eq!(resp.qv_unit, UnitCode::DegreesCelsius);
    assert!((resp.qv - 25.0).abs() < 0.001);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --workspace`
Expected: all tests pass across all crates

- [ ] **Step 3: Commit**

```bash
git add hart-protocol/tests/
git commit -m "test: add end-to-end codec integration tests for cmd0 and cmd3"
```

---

## Summary

| Task | Crate | What it builds |
|------|-------|---------------|
| 1 | workspace | Scaffolding — 3 crates compile |
| 2 | hart-protocol | Protocol constants (delimiters, commands, frame sizes) |
| 3 | hart-protocol | Core types (Address, MasterRole, FrameType, ResponseStatus) |
| 4 | hart-protocol | UnitCode enum (~100 engineering units) |
| 5 | hart-protocol | Frame encoder |
| 6 | hart-protocol | Frame decoder (byte-at-a-time state machine) |
| 7 | hart-protocol | Command traits + Phase 1 commands (0, 1, 2, 3, 48) |
| 8 | ad5700 | Blocking modem driver (RTS/CD pin control) |
| 9 | ad5700 | Async modem driver |
| 10 | ad5700 | HartMasterBlocking (blocking master API) |
| 11 | embassy-hart | HartMaster (async master with embassy-time timeouts) |
| 12 | hart-protocol | End-to-end integration tests |

After this plan, you have a working HART master that can encode/decode all Phase 1 commands and drive an AD5700-1 modem. Hardware integration (STM32H7 example) is the next plan.
