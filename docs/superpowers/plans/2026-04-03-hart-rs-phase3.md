# hart-rs Phase 3 Implementation Plan — Configuration (Write Commands)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add write commands (6, 17, 18, 19, 22, 38) to enable device configuration over HART — setting polling address, tags, messages, and clearing status flags.

**Architecture:** All changes in `hart-protocol` crate. Write commands follow the same `CommandRequest`/`CommandResponse` trait pattern. The master layers (`HartMasterBlocking`, `HartMaster`) automatically support them via `send_command<Req, Resp>()`.

**Tech Stack:** Same as Phase 1-2. No new dependencies.

**Prerequisite:** Phase 2 complete (Tasks 13-16).

**Spec:** `docs/superpowers/specs/2026-04-03-hart-rs-design.md` (Phase 3 section)

---

## File Structure (new files only)

```
hart-protocol/src/commands/
├── cmd6.rs                     (NEW: Write Polling Address)
├── cmd17.rs                    (NEW: Write Message)
├── cmd18.rs                    (NEW: Write Tag, Descriptor, Date)
├── cmd19.rs                    (NEW: Write Final Assembly Number)
├── cmd22.rs                    (NEW: Write Long Tag)
└── cmd38.rs                    (NEW: Reset Configuration Changed Flag)
```

---

### Task 17: Command 6 — Write Polling Address

**Files:**
- Create: `hart-protocol/src/commands/cmd6.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd6_request_encodes_address_and_mode() {
        let req = WritePollingAddressRequest {
            polling_address: 5,
            loop_current_mode: 0,
        };
        let mut buf = [0u8; 2];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
        assert_eq!(buf[0], 5);
        assert_eq!(buf[1], 0);
    }

    #[test]
    fn cmd6_response_decodes() {
        let data = [5, 0];
        let resp = WritePollingAddressResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 5);
        assert_eq!(resp.loop_current_mode, 0);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write implementation**

```rust
// hart-protocol/src/commands/cmd6.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct WritePollingAddressRequest {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

impl CommandRequest for WritePollingAddressRequest {
    const COMMAND_NUMBER: u8 = commands::WRITE_POLLING_ADDRESS;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 2 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[0] = self.polling_address;
        buf[1] = self.loop_current_mode;
        Ok(2)
    }
}

#[derive(Debug, Clone)]
pub struct WritePollingAddressResponse {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

impl CommandResponse for WritePollingAddressResponse {
    const COMMAND_NUMBER: u8 = commands::WRITE_POLLING_ADDRESS;
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
    fn cmd6_request_encodes_address_and_mode() {
        let req = WritePollingAddressRequest {
            polling_address: 5,
            loop_current_mode: 0,
        };
        let mut buf = [0u8; 2];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
        assert_eq!(buf[0], 5);
        assert_eq!(buf[1], 0);
    }

    #[test]
    fn cmd6_response_decodes() {
        let data = [5, 0];
        let resp = WritePollingAddressResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 5);
        assert_eq!(resp.loop_current_mode, 0);
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add: `pub mod cmd6;`

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add command 6 — write polling address"
```

---

### Task 18: Commands 17, 18, 19 — Write Message, Tag/Descriptor/Date, Final Assembly

**Files:**
- Create: `hart-protocol/src/commands/cmd17.rs`
- Create: `hart-protocol/src/commands/cmd18.rs`
- Create: `hart-protocol/src/commands/cmd19.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write tests**

```rust
// cmd17 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd17_request_encodes_packed_message() {
        let mut message = [b' '; 32];
        message[..5].copy_from_slice(b"HELLO");
        let req = WriteMessageRequest { message };
        let mut buf = [0u8; 24];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 24); // 32 chars packed = 24 bytes
    }

    #[test]
    fn cmd17_response_decodes() {
        let mut packed = [0u8; 24];
        crate::packed_string::encode_packed(b"HELLO                           ", &mut packed);
        let resp = WriteMessageResponse::decode_data(&packed).unwrap();
        assert!(resp.message.starts_with(b"HELLO"));
    }
}
```

```rust
// cmd18 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd18_request_encodes_tag_descriptor_date() {
        let req = WriteTagDescriptorDateRequest {
            tag: *b"NEWTAG  ",
            descriptor: *b"NEW DESCRIPTOR  ",
            day: 3,
            month: 4,
            year: 26,
        };
        let mut buf = [0u8; 21];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 21); // 6 + 12 + 3
        assert_eq!(buf[18], 3);  // day
        assert_eq!(buf[19], 4);  // month
        assert_eq!(buf[20], 26); // year
    }
}
```

```rust
// cmd19 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd19_request_encodes_assembly_number() {
        let req = WriteFinalAssemblyRequest { final_assembly_number: 0xAABBCC };
        let mut buf = [0u8; 3];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 3);
        assert_eq!(buf, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn cmd19_response_decodes() {
        let data = [0xAA, 0xBB, 0xCC];
        let resp = WriteFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0xAABBCC);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write implementations**

```rust
// hart-protocol/src/commands/cmd17.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::packed_string;
use super::{CommandRequest, CommandResponse};

pub struct WriteMessageRequest {
    pub message: [u8; 32],
}

impl CommandRequest for WriteMessageRequest {
    const COMMAND_NUMBER: u8 = commands::WRITE_MESSAGE;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 24 {
            return Err(EncodeError::BufferTooSmall);
        }
        let len = packed_string::encode_packed(&self.message, buf);
        Ok(len)
    }
}

#[derive(Debug, Clone)]
pub struct WriteMessageResponse {
    pub message: [u8; 32],
}

impl CommandResponse for WriteMessageResponse {
    const COMMAND_NUMBER: u8 = commands::WRITE_MESSAGE;
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
    fn cmd17_request_encodes_packed_message() {
        let mut message = [b' '; 32];
        message[..5].copy_from_slice(b"HELLO");
        let req = WriteMessageRequest { message };
        let mut buf = [0u8; 24];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 24);
    }

    #[test]
    fn cmd17_response_decodes() {
        let input = b"HELLO                           ";
        let mut packed = [0u8; 24];
        crate::packed_string::encode_packed(input, &mut packed);
        let resp = WriteMessageResponse::decode_data(&packed).unwrap();
        assert!(resp.message.starts_with(b"HELLO"));
    }
}
```

```rust
// hart-protocol/src/commands/cmd18.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use crate::packed_string;
use super::{CommandRequest, CommandResponse};

pub struct WriteTagDescriptorDateRequest {
    pub tag: [u8; 8],
    pub descriptor: [u8; 16],
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl CommandRequest for WriteTagDescriptorDateRequest {
    const COMMAND_NUMBER: u8 = commands::WRITE_TAG_DESCRIPTOR_DATE;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 21 {
            return Err(EncodeError::BufferTooSmall);
        }
        packed_string::encode_packed(&self.tag, &mut buf[0..6]);
        packed_string::encode_packed(&self.descriptor, &mut buf[6..18]);
        buf[18] = self.day;
        buf[19] = self.month;
        buf[20] = self.year;
        Ok(21)
    }
}

/// Response echoes back the same data.
pub use super::cmd13::ReadTagDescriptorDateResponse as WriteTagDescriptorDateResponse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd18_request_encodes_tag_descriptor_date() {
        let req = WriteTagDescriptorDateRequest {
            tag: *b"NEWTAG  ",
            descriptor: *b"NEW DESCRIPTOR  ",
            day: 3,
            month: 4,
            year: 26,
        };
        let mut buf = [0u8; 21];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 21);
        assert_eq!(buf[18], 3);
        assert_eq!(buf[19], 4);
        assert_eq!(buf[20], 26);
    }
}
```

```rust
// hart-protocol/src/commands/cmd19.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

pub struct WriteFinalAssemblyRequest {
    pub final_assembly_number: u32,
}

impl CommandRequest for WriteFinalAssemblyRequest {
    const COMMAND_NUMBER: u8 = commands::WRITE_FINAL_ASSEMBLY_NUMBER;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 3 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[0] = (self.final_assembly_number >> 16) as u8;
        buf[1] = (self.final_assembly_number >> 8) as u8;
        buf[2] = self.final_assembly_number as u8;
        Ok(3)
    }
}

#[derive(Debug, Clone)]
pub struct WriteFinalAssemblyResponse {
    pub final_assembly_number: u32,
}

impl CommandResponse for WriteFinalAssemblyResponse {
    const COMMAND_NUMBER: u8 = commands::WRITE_FINAL_ASSEMBLY_NUMBER;
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
    fn cmd19_request_encodes_assembly_number() {
        let req = WriteFinalAssemblyRequest { final_assembly_number: 0xAABBCC };
        let mut buf = [0u8; 3];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 3);
        assert_eq!(buf, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn cmd19_response_decodes() {
        let data = [0xAA, 0xBB, 0xCC];
        let resp = WriteFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0xAABBCC);
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add:
```rust
pub mod cmd17;
pub mod cmd18;
pub mod cmd19;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p hart-protocol`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add commands 17, 18, 19 — write message, tag/descriptor/date, final assembly"
```

---

### Task 19: Commands 22, 38 — Write Long Tag, Reset Config Changed Flag

**Files:**
- Create: `hart-protocol/src/commands/cmd22.rs`
- Create: `hart-protocol/src/commands/cmd38.rs`
- Modify: `hart-protocol/src/commands/mod.rs`

- [ ] **Step 1: Write tests**

```rust
// cmd22 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd22_request_encodes_long_tag() {
        let mut long_tag = [b' '; 32];
        long_tag[..11].copy_from_slice(b"VEGAPULS-21");
        let req = WriteLongTagRequest { long_tag };
        let mut buf = [0u8; 32];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 32);
        assert!(buf.starts_with(b"VEGAPULS-21"));
    }

    #[test]
    fn cmd22_response_decodes() {
        let mut data = [b' '; 32];
        data[..11].copy_from_slice(b"VEGAPULS-21");
        let resp = WriteLongTagResponse::decode_data(&data).unwrap();
        assert!(resp.long_tag.starts_with(b"VEGAPULS-21"));
    }
}
```

```rust
// cmd38 test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd38_request_encodes_empty() {
        let req = ResetConfigChangedRequest { configuration_change_counter: 0 };
        let mut buf = [0u8; 2];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
    }

    #[test]
    fn cmd38_response_decodes() {
        let data = [0x00, 0x05];
        let resp = ResetConfigChangedResponse::decode_data(&data).unwrap();
        assert_eq!(resp.configuration_change_counter, 5);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p hart-protocol`
Expected: FAIL

- [ ] **Step 3: Write implementations**

```rust
// hart-protocol/src/commands/cmd22.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

/// Command 22: Write Long Tag — 32 bytes plain ASCII (NOT packed).
pub struct WriteLongTagRequest {
    pub long_tag: [u8; 32],
}

impl CommandRequest for WriteLongTagRequest {
    const COMMAND_NUMBER: u8 = commands::WRITE_LONG_TAG;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 32 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[..32].copy_from_slice(&self.long_tag);
        Ok(32)
    }
}

#[derive(Debug, Clone)]
pub struct WriteLongTagResponse {
    pub long_tag: [u8; 32],
}

impl CommandResponse for WriteLongTagResponse {
    const COMMAND_NUMBER: u8 = commands::WRITE_LONG_TAG;
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
    fn cmd22_request_encodes_long_tag() {
        let mut long_tag = [b' '; 32];
        long_tag[..11].copy_from_slice(b"VEGAPULS-21");
        let req = WriteLongTagRequest { long_tag };
        let mut buf = [0u8; 32];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 32);
        assert!(buf.starts_with(b"VEGAPULS-21"));
    }

    #[test]
    fn cmd22_response_decodes() {
        let mut data = [b' '; 32];
        data[..11].copy_from_slice(b"VEGAPULS-21");
        let resp = WriteLongTagResponse::decode_data(&data).unwrap();
        assert!(resp.long_tag.starts_with(b"VEGAPULS-21"));
    }
}
```

```rust
// hart-protocol/src/commands/cmd38.rs
use crate::consts::commands;
use crate::error::{EncodeError, DecodeError};
use super::{CommandRequest, CommandResponse};

/// Command 38: Reset Configuration Changed Flag.
pub struct ResetConfigChangedRequest {
    pub configuration_change_counter: u16,
}

impl CommandRequest for ResetConfigChangedRequest {
    const COMMAND_NUMBER: u8 = commands::RESET_CONFIG_CHANGED;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 2 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[0] = (self.configuration_change_counter >> 8) as u8;
        buf[1] = self.configuration_change_counter as u8;
        Ok(2)
    }
}

#[derive(Debug, Clone)]
pub struct ResetConfigChangedResponse {
    pub configuration_change_counter: u16,
}

impl CommandResponse for ResetConfigChangedResponse {
    const COMMAND_NUMBER: u8 = commands::RESET_CONFIG_CHANGED;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(Self {
            configuration_change_counter: ((data[0] as u16) << 8) | (data[1] as u16),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd38_request_encodes() {
        let req = ResetConfigChangedRequest { configuration_change_counter: 0 };
        let mut buf = [0u8; 2];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
    }

    #[test]
    fn cmd38_response_decodes() {
        let data = [0x00, 0x05];
        let resp = ResetConfigChangedResponse::decode_data(&data).unwrap();
        assert_eq!(resp.configuration_change_counter, 5);
    }
}
```

- [ ] **Step 4: Update commands/mod.rs**

Add:
```rust
pub mod cmd22;
pub mod cmd38;
```

- [ ] **Step 5: Run all tests across workspace**

Run: `cargo test --workspace`
Expected: all tests pass

- [ ] **Step 6: Commit**

```bash
git add hart-protocol/src/
git commit -m "feat(hart-protocol): add commands 22, 38 — write long tag, reset config changed flag"
```

---

## Summary

| Task | What it builds |
|------|---------------|
| 17 | Command 6 — Write Polling Address |
| 18 | Commands 17, 18, 19 — Write Message, Tag/Descriptor/Date, Final Assembly Number |
| 19 | Commands 22, 38 — Write Long Tag, Reset Configuration Changed Flag |

After Phase 3, the HART master can read AND write all universal command data — device identity, configuration, measurement variables, diagnostics, tags, messages, and addresses. The full set of commands from the design spec is implemented.
