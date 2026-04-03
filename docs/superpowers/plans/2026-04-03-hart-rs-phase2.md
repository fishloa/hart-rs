# hart-rs Phase 2 Implementation Plan — Full Read-Only Access

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend hart-protocol with all read-only universal commands (7, 8, 9, 11, 12, 13, 14, 15, 16, 20), including packed string support for HART's 6-bit encoded tags and messages.

**Architecture:** All changes are in the `hart-protocol` crate. No changes to `ad5700` or `embassy-hart` — those crates use the generic `send_command<Req, Resp>()` method which automatically works with new command types.

**Tech Stack:** Same as Phase 1. No new dependencies.

**Prerequisite:** Phase 1 complete (Tasks 1-12).

**Spec:** `docs/superpowers/specs/2026-04-03-hart-rs-design.md` (Phase 2 section)

---

## File Structure (new/modified files only)

```
hart-protocol/src/
├── packed_string.rs                (NEW: 6-bit HART string encode/decode)
├── commands/
│   ├── cmd7.rs                     (NEW: Read Loop Configuration)
│   ├── cmd8.rs                     (NEW: Read Dynamic Variable Classifications)
│   ├── cmd9.rs                     (NEW: Read Device Variables with Status)
│   ├── cmd11.rs                    (NEW: Read Unique ID by Tag)
│   ├── cmd12.rs                    (NEW: Read Message)
│   ├── cmd13.rs                    (NEW: Read Tag, Descriptor, Date)
│   ├── cmd14.rs                    (NEW: Read PV Transducer Info)
│   ├── cmd15.rs                    (NEW: Read Device Information)
│   ├── cmd16.rs                    (NEW: Read Final Assembly Number)
│   └── cmd20.rs                    (NEW: Read Long Tag)
```

---

### Task 13: Packed String Encode/Decode

HART uses a 6-bit character encoding for tags and messages. 4 characters are packed into 3 bytes. The character set is: `@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_ !"#$%&'()*+,-./0123456789:;<=>?`
(codes 0-63 map to ASCII 0x40-0x7F, wrapping around).

**Files:**
- Create: `hart-protocol/src/packed_string.rs`
- Modify: `hart-protocol/src/lib.rs`

- [ ] **Step 1: Write packed string tests**

```rust
// hart-protocol/src/packed_string.rs — append at bottom after Step 3
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty() {
        let mut out = [0u8; 32];
        let len = decode_packed(&[], &mut out);
        assert_eq!(len, 0);
    }

    #[test]
    fn encode_decode_roundtrip_tag() {
        // "SENSOR01" — 8 chars = 6 bytes packed
        let input = b"SENSOR01";
        let mut packed = [0u8; 6];
        let packed_len = encode_packed(input, &mut packed);
        assert_eq!(packed_len, 6);

        let mut output = [0u8; 8];
        let out_len = decode_packed(&packed[..packed_len], &mut output);
        assert_eq!(out_len, 8);
        assert_eq!(&output[..out_len], b"SENSOR01");
    }

    #[test]
    fn encode_decode_roundtrip_spaces() {
        // Spaces are valid in packed strings (code 0x20 => packed 0)
        let input = b"HI THERE";
        let mut packed = [0u8; 6];
        let packed_len = encode_packed(input, &mut packed);

        let mut output = [0u8; 8];
        let out_len = decode_packed(&packed[..packed_len], &mut output);
        assert_eq!(&output[..out_len], b"HI THERE");
    }

    #[test]
    fn encode_decode_32char_message() {
        // Command 12 uses 32-char packed messages = 24 bytes
        let msg = b"HART TEST MESSAGE 12345678901234"; // 32 chars (padded with spaces below)
        let input = b"HART TEST MESSAGE 1234567890123 "; // exactly 32
        let mut packed = [0u8; 24];
        let packed_len = encode_packed(input, &mut packed);
        assert_eq!(packed_len, 24);

        let mut output = [0u8; 32];
        let out_len = decode_packed(&packed[..packed_len], &mut output);
        assert_eq!(out_len, 32);
    }

    #[test]
    fn decode_3_bytes_to_4_chars() {
        // Manual verification: pack "TEST"
        // T=0x14, E=0x05, S=0x13, T=0x14
        // Byte 0: (T<<2) | (E>>4) = (0x14<<2) | (0x05>>4) = 0x50 | 0x00 = 0x50
        // Byte 1: (E<<4) | (S>>2) = (0x05<<4) | (0x13>>2) = 0x50 | 0x04 = 0x54
        // Byte 2: (S<<6) | T = (0x13<<6) | 0x14 = 0xC0 | 0x14 = 0xD4
        let mut packed = [0u8; 3];
        encode_packed(b"TEST", &mut packed);

        let mut out = [0u8; 4];
        let len = decode_packed(&packed, &mut out);
        assert_eq!(len, 4);
        assert_eq!(&out, b"TEST");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL — `packed_string` module not found

- [ ] **Step 3: Write packed string implementation**

```rust
// hart-protocol/src/packed_string.rs

/// Convert ASCII character to 6-bit HART packed code.
/// HART packed charset: space is 0x20 => code 0, '!' => 1, ... '?' => 31,
/// '@' => 32, 'A' => 33, ... 'Z' => 58, etc.
/// Effectively: code = (ascii & 0x3F)
fn char_to_packed(c: u8) -> u8 {
    c & 0x3F
}

/// Convert 6-bit HART packed code to ASCII character.
/// Codes 0-31 map to ASCII 0x20-0x3F (space through '?')
/// Codes 32-63 map to ASCII 0x40-0x7F ('@' through DEL, but DEL unused)
fn packed_to_char(code: u8) -> u8 {
    let code = code & 0x3F;
    if code < 32 {
        code + 0x20
    } else {
        code + 0x20
    }
}

/// Decode HART 6-bit packed string from `src` into ASCII bytes in `dst`.
/// Every 3 src bytes produce 4 dst characters.
/// Returns number of characters written to dst.
pub fn decode_packed(src: &[u8], dst: &mut [u8]) -> usize {
    let mut si = 0;
    let mut di = 0;
    while si + 2 < src.len() && di + 3 < dst.len() {
        let b0 = src[si];
        let b1 = src[si + 1];
        let b2 = src[si + 2];

        dst[di] = packed_to_char(b0 >> 2);
        dst[di + 1] = packed_to_char(((b0 & 0x03) << 4) | (b1 >> 4));
        dst[di + 2] = packed_to_char(((b1 & 0x0F) << 2) | (b2 >> 6));
        dst[di + 3] = packed_to_char(b2 & 0x3F);

        si += 3;
        di += 4;
    }
    // Handle remaining bytes (1 or 2 leftover)
    if si < src.len() && di < dst.len() {
        dst[di] = packed_to_char(src[si] >> 2);
        di += 1;
        if si + 1 < src.len() && di < dst.len() {
            dst[di] = packed_to_char(((src[si] & 0x03) << 4) | (src[si + 1] >> 4));
            di += 1;
            if di < dst.len() {
                dst[di] = packed_to_char((src[si + 1] & 0x0F) << 2);
                di += 1;
            }
        } else if di < dst.len() {
            dst[di] = packed_to_char((src[si] & 0x03) << 4);
            di += 1;
        }
    }
    di
}

/// Encode ASCII string `src` into HART 6-bit packed bytes in `dst`.
/// Every 4 src characters produce 3 dst bytes.
/// Returns number of bytes written to dst.
pub fn encode_packed(src: &[u8], dst: &mut [u8]) -> usize {
    let mut si = 0;
    let mut di = 0;
    while si + 3 < src.len() && di + 2 < dst.len() {
        let c0 = char_to_packed(src[si]);
        let c1 = char_to_packed(src[si + 1]);
        let c2 = char_to_packed(src[si + 2]);
        let c3 = char_to_packed(src[si + 3]);

        dst[di] = (c0 << 2) | (c1 >> 4);
        dst[di + 1] = (c1 << 4) | (c2 >> 2);
        dst[di + 2] = (c2 << 6) | c3;

        si += 4;
        di += 3;
    }
    // Handle remaining 1-3 characters
    if si < src.len() && di < dst.len() {
        let c0 = char_to_packed(src[si]);
        if si + 1 < src.len() {
            let c1 = char_to_packed(src[si + 1]);
            dst[di] = (c0 << 2) | (c1 >> 4);
            di += 1;
            if si + 2 < src.len() && di < dst.len() {
                let c2 = char_to_packed(src[si + 2]);
                dst[di] = (c1 << 4) | (c2 >> 2);
                di += 1;
                if di < dst.len() {
                    dst[di] = c2 << 6;
                    di += 1;
                }
            } else if di < dst.len() {
                dst[di] = c1 << 4;
                di += 1;
            }
        } else {
            dst[di] = c0 << 2;
            di += 1;
        }
    }
    di
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_empty() {
        let mut out = [0u8; 32];
        let len = decode_packed(&[], &mut out);
        assert_eq!(len, 0);
    }

    #[test]
    fn encode_decode_roundtrip_tag() {
        let input = b"SENSOR01";
        let mut packed = [0u8; 6];
        let packed_len = encode_packed(input, &mut packed);
        assert_eq!(packed_len, 6);

        let mut output = [0u8; 8];
        let out_len = decode_packed(&packed[..packed_len], &mut output);
        assert_eq!(out_len, 8);
        assert_eq!(&output[..out_len], b"SENSOR01");
    }

    #[test]
    fn encode_decode_roundtrip_spaces() {
        let input = b"HI THERE";
        let mut packed = [0u8; 6];
        let packed_len = encode_packed(input, &mut packed);

        let mut output = [0u8; 8];
        let out_len = decode_packed(&packed[..packed_len], &mut output);
        assert_eq!(&output[..out_len], b"HI THERE");
    }

    #[test]
    fn decode_3_bytes_to_4_chars() {
        let mut packed = [0u8; 3];
        encode_packed(b"TEST", &mut packed);

        let mut out = [0u8; 4];
        let len = decode_packed(&packed, &mut out);
        assert_eq!(len, 4);
        assert_eq!(&out, b"TEST");
    }
}
```

- [ ] **Step 4: Wire into lib.rs**

Add `pub mod packed_string;` to `hart-protocol/src/lib.rs`.

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add HART 6-bit packed string encode/decode"
```

---

### Task 14: Commands 7, 8, 9 — Loop Config, Variable Classifications, Variables with Status

**Files:**
- Create: `hart-protocol/src/commands/cmd7.rs`
- Create: `hart-protocol/src/commands/cmd8.rs`
- Create: `hart-protocol/src/commands/cmd9.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write tests for cmd7, cmd8, cmd9**

```rust
// hart-protocol/src/commands/cmd7.rs — tests at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd7_response_decodes() {
        // polling_address(1) + loop_current_mode(1) = 2 bytes
        let data = [0x05, 0x00]; // address=5, mode=0 (enabled)
        let resp = ReadLoopConfigResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 5);
        assert_eq!(resp.loop_current_mode, 0);
    }
}
```

```rust
// hart-protocol/src/commands/cmd8.rs — tests at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd8_response_decodes() {
        let data = [69, 69, 64, 64]; // PV=length(69), SV=length(69), TV=temp(64), QV=temp(64)
        let resp = ReadDynamicVarClassResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_classification, 69);
        assert_eq!(resp.sv_classification, 69);
        assert_eq!(resp.tv_classification, 64);
        assert_eq!(resp.qv_classification, 64);
    }
}
```

```rust
// hart-protocol/src/commands/cmd9.rs — tests at bottom
#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::UnitCode;

    #[test]
    fn cmd9_request_encodes_slot_codes() {
        let req = ReadDeviceVarsRequest {
            slot_codes: heapless::Vec::from_slice(&[0, 1, 2, 3]).unwrap(),
        };
        let mut buf = [0u8; 8];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 4);
        assert_eq!(&buf[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn cmd9_response_decodes_single_var() {
        // Per variable: device_var_code(1) + classification(1) + unit(1) + value(4) + status(1) = 8 bytes
        let mut data = [0u8; 8];
        data[0] = 0;  // device variable code
        data[1] = 69; // classification: length
        data[2] = 45; // unit: meters
        data[3..7].copy_from_slice(&1.23_f32.to_be_bytes());
        data[7] = 0x00; // status: good

        let resp = ReadDeviceVarsResponse::decode_data(&data).unwrap();
        assert_eq!(resp.variables.len(), 1);
        assert_eq!(resp.variables[0].unit, UnitCode::Meters);
        assert!((resp.variables[0].value - 1.23).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write command implementations**

```rust
// hart-protocol/src/commands/cmd7.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct ReadLoopConfigRequest;

impl CommandRequest for ReadLoopConfigRequest {
    const COMMAND_NUMBER: u8 = commands::READ_LOOP_CONFIG;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

#[derive(Debug, Clone)]
pub struct ReadLoopConfigResponse {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

impl CommandResponse for ReadLoopConfigResponse {
    const COMMAND_NUMBER: u8 = commands::READ_LOOP_CONFIG;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            polling_address: data[0],
            loop_current_mode: data[1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd7_response_decodes() {
        let data = [0x05, 0x00];
        let resp = ReadLoopConfigResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 5);
        assert_eq!(resp.loop_current_mode, 0);
    }
}
```

```rust
// hart-protocol/src/commands/cmd8.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct ReadDynamicVarClassRequest;

impl CommandRequest for ReadDynamicVarClassRequest {
    const COMMAND_NUMBER: u8 = commands::READ_DYNAMIC_VAR_CLASS;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

#[derive(Debug, Clone)]
pub struct ReadDynamicVarClassResponse {
    pub pv_classification: u8,
    pub sv_classification: u8,
    pub tv_classification: u8,
    pub qv_classification: u8,
}

impl CommandResponse for ReadDynamicVarClassResponse {
    const COMMAND_NUMBER: u8 = commands::READ_DYNAMIC_VAR_CLASS;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 4 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            pv_classification: data[0],
            sv_classification: data[1],
            tv_classification: data[2],
            qv_classification: data[3],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd8_response_decodes() {
        let data = [69, 69, 64, 64];
        let resp = ReadDynamicVarClassResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_classification, 69);
        assert_eq!(resp.tv_classification, 64);
    }
}
```

```rust
// hart-protocol/src/commands/cmd9.rs
use heapless::Vec;
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

/// Command 9 request: specify which device variable slots to read (up to 8).
pub struct ReadDeviceVarsRequest {
    pub slot_codes: Vec<u8, 8>,
}

impl CommandRequest for ReadDeviceVarsRequest {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_VARS_WITH_STATUS;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < self.slot_codes.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[..self.slot_codes.len()].copy_from_slice(&self.slot_codes);
        Ok(self.slot_codes.len())
    }
}

/// A single device variable with status from Command 9 response.
#[derive(Debug, Clone)]
pub struct DeviceVariable {
    pub device_var_code: u8,
    pub classification: u8,
    pub unit: UnitCode,
    pub value: f32,
    pub status: u8,
}

/// Command 9 response: variable-length list of device variables.
#[derive(Debug, Clone)]
pub struct ReadDeviceVarsResponse {
    pub variables: Vec<DeviceVariable, 8>,
}

impl CommandResponse for ReadDeviceVarsResponse {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_VARS_WITH_STATUS;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        // Each variable: code(1) + class(1) + unit(1) + value(4) + status(1) = 8 bytes
        let mut vars = Vec::new();
        let mut pos = 0;
        while pos + 8 <= data.len() && !vars.is_full() {
            let dv = DeviceVariable {
                device_var_code: data[pos],
                classification: data[pos + 1],
                unit: UnitCode::from_u8(data[pos + 2]),
                value: f32::from_be_bytes([
                    data[pos + 3],
                    data[pos + 4],
                    data[pos + 5],
                    data[pos + 6],
                ]),
                status: data[pos + 7],
            };
            let _ = vars.push(dv);
            pos += 8;
        }
        Ok(Self { variables: vars })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd9_request_encodes_slot_codes() {
        let req = ReadDeviceVarsRequest {
            slot_codes: Vec::from_slice(&[0, 1, 2, 3]).unwrap(),
        };
        let mut buf = [0u8; 8];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 4);
        assert_eq!(&buf[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn cmd9_response_decodes_single_var() {
        let mut data = [0u8; 8];
        data[0] = 0;
        data[1] = 69;
        data[2] = 45;
        data[3..7].copy_from_slice(&1.23_f32.to_be_bytes());
        data[7] = 0x00;

        let resp = ReadDeviceVarsResponse::decode_data(&data).unwrap();
        assert_eq!(resp.variables.len(), 1);
        assert_eq!(resp.variables[0].unit, UnitCode::Meters);
        assert!((resp.variables[0].value - 1.23).abs() < 0.001);
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add to `hart-protocol/src/commands/mod.rs`:
```rust
pub mod cmd7;
pub mod cmd8;
pub mod cmd9;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add commands 7, 8, 9 — loop config, var classifications, device vars with status"
```

---

### Task 15: Commands 11, 12, 13 — Tag Lookup, Message, Tag/Descriptor/Date

These commands use packed strings.

**Files:**
- Create: `hart-protocol/src/commands/cmd11.rs`
- Create: `hart-protocol/src/commands/cmd12.rs`
- Create: `hart-protocol/src/commands/cmd13.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write tests**

```rust
// cmd11 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd11_request_encodes_packed_tag() {
        let req = ReadUniqueIdByTagRequest { tag: *b"SENSOR01" };
        let mut buf = [0u8; 6];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 6); // 8 chars packed into 6 bytes
    }
}
```

```rust
// cmd12 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd12_response_decodes_message() {
        // 24 bytes of packed data = 32 characters
        let mut packed = [0u8; 24];
        crate::packed_string::encode_packed(b"HELLO WORLD TEST MESSAGE", &mut packed);
        let resp = ReadMessageResponse::decode_data(&packed).unwrap();
        assert!(resp.message.starts_with(b"HELLO WORLD"));
    }
}
```

```rust
// cmd13 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd13_response_decodes_tag_descriptor_date() {
        // tag: 6 packed bytes (8 chars) + descriptor: 12 packed bytes (16 chars) + date: 3 bytes
        let mut data = [0u8; 21];
        crate::packed_string::encode_packed(b"MYTAG   ", &mut data[0..6]);
        crate::packed_string::encode_packed(b"MY DESCRIPTOR   ", &mut data[6..18]);
        data[18] = 1;  // day
        data[19] = 4;  // month
        data[20] = 26; // year (from 1900, so 2026)

        let resp = ReadTagDescriptorDateResponse::decode_data(&data).unwrap();
        assert!(resp.tag.starts_with(b"MYTAG"));
        assert!(resp.descriptor.starts_with(b"MY DESC"));
        assert_eq!(resp.day, 1);
        assert_eq!(resp.month, 4);
        assert_eq!(resp.year, 26);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write command implementations**

```rust
// hart-protocol/src/commands/cmd11.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::packed_string;
use super::{CommandRequest, CommandResponse};

/// Command 11: Read Unique Identifier by Tag — request.
pub struct ReadUniqueIdByTagRequest {
    /// 8-character tag (ASCII, will be packed to 6 bytes).
    pub tag: [u8; 8],
}

impl CommandRequest for ReadUniqueIdByTagRequest {
    const COMMAND_NUMBER: u8 = commands::READ_UNIQUE_ID_BY_TAG;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 6 {
            return Err(EncodeError::BufferTooSmall);
        }
        let len = packed_string::encode_packed(&self.tag, buf);
        Ok(len)
    }
}

/// Command 11 response is identical to Command 0 response.
pub use super::cmd0::ReadDeviceIdResponse as ReadUniqueIdByTagResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd11_request_encodes_packed_tag() {
        let req = ReadUniqueIdByTagRequest { tag: *b"SENSOR01" };
        let mut buf = [0u8; 6];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 6);
    }
}
```

```rust
// hart-protocol/src/commands/cmd12.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::packed_string;
use super::{CommandRequest, CommandResponse};

pub struct ReadMessageRequest;

impl CommandRequest for ReadMessageRequest {
    const COMMAND_NUMBER: u8 = commands::READ_MESSAGE;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 12 response: 32-character packed message (24 bytes packed).
#[derive(Debug, Clone)]
pub struct ReadMessageResponse {
    pub message: [u8; 32],
}

impl CommandResponse for ReadMessageResponse {
    const COMMAND_NUMBER: u8 = commands::READ_MESSAGE;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 24 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut message = [b' '; 32];
        packed_string::decode_packed(&data[..24], &mut message);
        Ok(Self { message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd12_response_decodes_message() {
        let input = b"HELLO WORLD TEST MSG 123456ABCDEF";
        let mut packed = [0u8; 24];
        crate::packed_string::encode_packed(input, &mut packed);
        let resp = ReadMessageResponse::decode_data(&packed).unwrap();
        assert!(resp.message.starts_with(b"HELLO WORLD"));
    }
}
```

```rust
// hart-protocol/src/commands/cmd13.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::packed_string;
use super::{CommandRequest, CommandResponse};

pub struct ReadTagDescriptorDateRequest;

impl CommandRequest for ReadTagDescriptorDateRequest {
    const COMMAND_NUMBER: u8 = commands::READ_TAG_DESCRIPTOR_DATE;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 13 response: tag (8 chars/6 packed) + descriptor (16 chars/12 packed) + date (3 bytes).
#[derive(Debug, Clone)]
pub struct ReadTagDescriptorDateResponse {
    pub tag: [u8; 8],
    pub descriptor: [u8; 16],
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl CommandResponse for ReadTagDescriptorDateResponse {
    const COMMAND_NUMBER: u8 = commands::READ_TAG_DESCRIPTOR_DATE;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        // 6 (tag packed) + 12 (descriptor packed) + 3 (date) = 21 bytes
        if data.len() < 21 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut tag = [b' '; 8];
        packed_string::decode_packed(&data[0..6], &mut tag);
        let mut descriptor = [b' '; 16];
        packed_string::decode_packed(&data[6..18], &mut descriptor);
        Ok(Self {
            tag,
            descriptor,
            day: data[18],
            month: data[19],
            year: data[20],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd13_response_decodes_tag_descriptor_date() {
        let mut data = [0u8; 21];
        crate::packed_string::encode_packed(b"MYTAG   ", &mut data[0..6]);
        crate::packed_string::encode_packed(b"MY DESCRIPTOR   ", &mut data[6..18]);
        data[18] = 1;
        data[19] = 4;
        data[20] = 26;

        let resp = ReadTagDescriptorDateResponse::decode_data(&data).unwrap();
        assert!(resp.tag.starts_with(b"MYTAG"));
        assert_eq!(resp.day, 1);
        assert_eq!(resp.month, 4);
        assert_eq!(resp.year, 26);
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add:
```rust
pub mod cmd11;
pub mod cmd12;
pub mod cmd13;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add commands 11, 12, 13 — tag lookup, message, tag/descriptor/date"
```

---

### Task 16: Commands 14, 15, 16, 20 — Transducer Info, Device Info, Assembly Number, Long Tag

**Files:**
- Create: `hart-protocol/src/commands/cmd14.rs`
- Create: `hart-protocol/src/commands/cmd15.rs`
- Create: `hart-protocol/src/commands/cmd16.rs`
- Create: `hart-protocol/src/commands/cmd20.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write tests for all four commands**

```rust
// cmd14 test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::UnitCode;

    #[test]
    fn cmd14_response_decodes() {
        let mut data = [0u8; 16];
        // transducer_serial(3) + unit(1) + upper_limit(4) + lower_limit(4) + minimum_span(4)
        data[0..3].copy_from_slice(&[0x11, 0x22, 0x33]); // serial
        data[3] = 45; // meters
        data[4..8].copy_from_slice(&20.0_f32.to_be_bytes());
        data[8..12].copy_from_slice(&0.0_f32.to_be_bytes());
        data[12..16].copy_from_slice(&0.1_f32.to_be_bytes());

        let resp = ReadPvTransducerInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.unit, UnitCode::Meters);
        assert!((resp.upper_limit - 20.0).abs() < 0.001);
    }
}
```

```rust
// cmd15 test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::UnitCode;

    #[test]
    fn cmd15_response_decodes() {
        let mut data = [0u8; 18];
        // alarm_selection(1) + transfer_function(1) + unit(1) +
        // upper_range(4) + lower_range(4) + damping(4) + write_protect(1) +
        // private_label_distributor(1) + final_assembly_number(3) — but only first 14 required
        data[0] = 0x00; // alarm selection
        data[1] = 0x00; // transfer function: linear
        data[2] = 57;   // percent
        data[3..7].copy_from_slice(&100.0_f32.to_be_bytes()); // upper range
        data[7..11].copy_from_slice(&0.0_f32.to_be_bytes());  // lower range
        data[11..15].copy_from_slice(&0.5_f32.to_be_bytes()); // damping
        data[15] = 251; // write protect: not implemented
        data[16] = 0x00; // private label
        data[17] = 0x00; // (partial final assembly — test just checks decode)

        let resp = ReadDeviceInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_unit, UnitCode::Percent);
        assert!((resp.upper_range - 100.0).abs() < 0.001);
        assert!((resp.damping - 0.5).abs() < 0.001);
    }
}
```

```rust
// cmd16 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd16_response_decodes() {
        let data = [0x11, 0x22, 0x33];
        let resp = ReadFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0x112233);
    }
}
```

```rust
// cmd20 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd20_response_decodes_long_tag() {
        let resp = ReadLongTagResponse::decode_data(b"VEGAPULS-21-LEVEL-SENSOR-A      ").unwrap();
        assert!(resp.long_tag.starts_with(b"VEGAPULS-21"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write command implementations**

```rust
// hart-protocol/src/commands/cmd14.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

pub struct ReadPvTransducerInfoRequest;

impl CommandRequest for ReadPvTransducerInfoRequest {
    const COMMAND_NUMBER: u8 = commands::READ_PV_TRANSDUCER_INFO;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

#[derive(Debug, Clone)]
pub struct ReadPvTransducerInfoResponse {
    pub transducer_serial: u32,
    pub unit: UnitCode,
    pub upper_limit: f32,
    pub lower_limit: f32,
    pub minimum_span: f32,
}

impl CommandResponse for ReadPvTransducerInfoResponse {
    const COMMAND_NUMBER: u8 = commands::READ_PV_TRANSDUCER_INFO;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 16 {
            return Err(DecodeError::BufferTooShort);
        }
        let serial = ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32);
        Ok(Self {
            transducer_serial: serial,
            unit: UnitCode::from_u8(data[3]),
            upper_limit: f32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            lower_limit: f32::from_be_bytes([data[8], data[9], data[10], data[11]]),
            minimum_span: f32::from_be_bytes([data[12], data[13], data[14], data[15]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd14_response_decodes() {
        let mut data = [0u8; 16];
        data[0..3].copy_from_slice(&[0x11, 0x22, 0x33]);
        data[3] = 45;
        data[4..8].copy_from_slice(&20.0_f32.to_be_bytes());
        data[8..12].copy_from_slice(&0.0_f32.to_be_bytes());
        data[12..16].copy_from_slice(&0.1_f32.to_be_bytes());

        let resp = ReadPvTransducerInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.unit, UnitCode::Meters);
        assert!((resp.upper_limit - 20.0).abs() < 0.001);
    }
}
```

```rust
// hart-protocol/src/commands/cmd15.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

pub struct ReadDeviceInfoRequest;

impl CommandRequest for ReadDeviceInfoRequest {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_INFO;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

#[derive(Debug, Clone)]
pub struct ReadDeviceInfoResponse {
    pub pv_alarm_selection: u8,
    pub pv_transfer_function: u8,
    pub pv_unit: UnitCode,
    pub upper_range: f32,
    pub lower_range: f32,
    pub damping: f32,
    pub write_protect: u8,
    pub private_label_distributor: u8,
}

impl CommandResponse for ReadDeviceInfoResponse {
    const COMMAND_NUMBER: u8 = commands::READ_DEVICE_INFO;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 16 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            pv_alarm_selection: data[0],
            pv_transfer_function: data[1],
            pv_unit: UnitCode::from_u8(data[2]),
            upper_range: f32::from_be_bytes([data[3], data[4], data[5], data[6]]),
            lower_range: f32::from_be_bytes([data[7], data[8], data[9], data[10]]),
            damping: f32::from_be_bytes([data[11], data[12], data[13], data[14]]),
            write_protect: data[15],
            private_label_distributor: if data.len() > 16 { data[16] } else { 0 },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd15_response_decodes() {
        let mut data = [0u8; 18];
        data[0] = 0x00;
        data[1] = 0x00;
        data[2] = 57;
        data[3..7].copy_from_slice(&100.0_f32.to_be_bytes());
        data[7..11].copy_from_slice(&0.0_f32.to_be_bytes());
        data[11..15].copy_from_slice(&0.5_f32.to_be_bytes());
        data[15] = 251;

        let resp = ReadDeviceInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_unit, UnitCode::Percent);
        assert!((resp.upper_range - 100.0).abs() < 0.001);
    }
}
```

```rust
// hart-protocol/src/commands/cmd16.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct ReadFinalAssemblyRequest;

impl CommandRequest for ReadFinalAssemblyRequest {
    const COMMAND_NUMBER: u8 = commands::READ_FINAL_ASSEMBLY_NUMBER;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

#[derive(Debug, Clone)]
pub struct ReadFinalAssemblyResponse {
    pub final_assembly_number: u32,
}

impl CommandResponse for ReadFinalAssemblyResponse {
    const COMMAND_NUMBER: u8 = commands::READ_FINAL_ASSEMBLY_NUMBER;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 3 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            final_assembly_number: ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd16_response_decodes() {
        let data = [0x11, 0x22, 0x33];
        let resp = ReadFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0x112233);
    }
}
```

```rust
// hart-protocol/src/commands/cmd20.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct ReadLongTagRequest;

impl CommandRequest for ReadLongTagRequest {
    const COMMAND_NUMBER: u8 = commands::READ_LONG_TAG;
    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> { Ok(0) }
}

/// Command 20 response: 32-byte long tag (plain ASCII, NOT packed).
#[derive(Debug, Clone)]
pub struct ReadLongTagResponse {
    pub long_tag: [u8; 32],
}

impl CommandResponse for ReadLongTagResponse {
    const COMMAND_NUMBER: u8 = commands::READ_LONG_TAG;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 32 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut long_tag = [0u8; 32];
        long_tag.copy_from_slice(&data[..32]);
        Ok(Self { long_tag })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd20_response_decodes_long_tag() {
        let resp = ReadLongTagResponse::decode_data(b"VEGAPULS-21-LEVEL-SENSOR-A      ").unwrap();
        assert!(resp.long_tag.starts_with(b"VEGAPULS-21"));
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add:
```rust
pub mod cmd14;
pub mod cmd15;
pub mod cmd16;
pub mod cmd20;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add commands 14, 15, 16, 20 — transducer info, device info, assembly number, long tag"
```

---

## Summary

| Task | What it builds |
|------|---------------|
| 13 | Packed string encode/decode (6-bit HART strings) |
| 14 | Commands 7, 8, 9 — loop config, variable classifications, device vars with status |
| 15 | Commands 11, 12, 13 — tag lookup, message, tag/descriptor/date (use packed strings) |
| 16 | Commands 14, 15, 16, 20 — transducer info, device info, assembly number, long tag |

After Phase 2, the HART master can read all device identity, configuration, measurement, and diagnostic data from any HART 7 device.
