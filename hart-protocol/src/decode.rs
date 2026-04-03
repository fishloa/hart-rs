use heapless::Vec;

use crate::consts::{delimiter, MIN_PREAMBLE_COUNT, PREAMBLE_BYTE};
use crate::error::DecodeError;
use crate::types::{Address, FrameType};

/// A fully decoded raw HART frame.
#[derive(Debug, Clone, PartialEq)]
pub struct RawFrame {
    pub frame_type: FrameType,
    pub address: Address,
    pub command: u8,
    /// Data payload (excludes status bytes — those are in the raw data for responses
    /// since this is a raw frame layer; upper layers strip status bytes).
    pub data: Vec<u8, 256>,
}

/// Internal state-machine states for the HART decoder.
#[derive(Debug, Clone)]
enum State {
    /// Waiting for preamble bytes.
    Hunting,
    /// Counting preamble 0xFF bytes; inner value is how many we've seen.
    Preamble(u8),
    /// Reading address bytes; inner value is how many bytes remain.
    Address(u8),
    /// Next byte is the command field.
    Command,
    /// Next byte is the byte-count field.
    ByteCount,
    /// Reading data bytes; inner value is how many bytes remain.
    Data(u8),
    /// Next byte is the checksum.
    Checksum,
}

/// Byte-at-a-time HART frame decoder (state machine).
pub struct Decoder {
    state: State,
    frame_type: Option<FrameType>,
    is_long: bool,
    /// Partial address bytes accumulated so far (up to 5).
    addr_buf: [u8; 5],
    addr_len: u8,
    addr_idx: u8,
    command: u8,
    /// Checksum accumulated from delimiter through last data byte.
    checksum: u8,
    data: Vec<u8, 256>,
}

impl Decoder {
    pub fn new() -> Self {
        Decoder {
            state: State::Hunting,
            frame_type: None,
            is_long: false,
            addr_buf: [0u8; 5],
            addr_len: 0,
            addr_idx: 0,
            command: 0,
            checksum: 0,
            data: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = State::Hunting;
        self.frame_type = None;
        self.is_long = false;
        self.addr_buf = [0u8; 5];
        self.addr_len = 0;
        self.addr_idx = 0;
        self.command = 0;
        self.checksum = 0;
        self.data.clear();
    }

    /// Feed one byte into the decoder.
    ///
    /// Returns:
    /// - `Ok(None)` — frame not yet complete, keep feeding bytes.
    /// - `Ok(Some(frame))` — a complete, checksum-valid frame was decoded.
    /// - `Err(e)` — a decode error occurred (state is reset automatically).
    pub fn feed(&mut self, byte: u8) -> Result<Option<RawFrame>, DecodeError> {
        match &self.state.clone() {
            State::Hunting => {
                if byte == PREAMBLE_BYTE {
                    self.state = State::Preamble(1);
                }
                // Non-preamble byte: stay Hunting, ignore it
                Ok(None)
            }

            State::Preamble(count) => {
                if byte == PREAMBLE_BYTE {
                    self.state = State::Preamble(count + 1);
                    Ok(None)
                } else if *count >= MIN_PREAMBLE_COUNT {
                    // This byte is the delimiter
                    self.process_delimiter(byte)
                } else {
                    // Not enough preambles, this byte is not a valid delimiter; reset
                    self.reset();
                    Ok(None)
                }
            }

            State::Address(_remaining) => {
                let idx = self.addr_idx as usize;
                self.addr_buf[idx] = byte;
                self.checksum ^= byte;
                self.addr_idx += 1;

                if self.addr_idx >= self.addr_len {
                    self.state = State::Command;
                } else {
                    let remaining = self.addr_len - self.addr_idx;
                    self.state = State::Address(remaining);
                }
                Ok(None)
            }

            State::Command => {
                self.command = byte;
                self.checksum ^= byte;
                self.state = State::ByteCount;
                Ok(None)
            }

            State::ByteCount => {
                let count = byte;
                self.checksum ^= byte;
                if count == 0 {
                    self.state = State::Checksum;
                } else {
                    self.state = State::Data(count);
                }
                Ok(None)
            }

            State::Data(remaining) => {
                // SAFETY: data Vec has capacity 256, byte_count <= 255
                let _ = self.data.push(byte);
                self.checksum ^= byte;

                let new_remaining = remaining - 1;
                if new_remaining == 0 {
                    self.state = State::Checksum;
                } else {
                    self.state = State::Data(new_remaining);
                }
                Ok(None)
            }

            State::Checksum => {
                let received = byte;
                let computed = self.checksum;
                if received != computed {
                    let err = DecodeError::ChecksumMismatch {
                        expected: computed,
                        actual: received,
                    };
                    self.reset();
                    return Err(err);
                }

                // Build the address
                let addr_result = Address::decode(&self.addr_buf[..self.addr_len as usize], self.is_long);
                let address = match addr_result {
                    Ok((addr, _)) => addr,
                    Err(e) => {
                        self.reset();
                        return Err(e);
                    }
                };

                let frame = RawFrame {
                    frame_type: self.frame_type.clone().unwrap(),
                    address,
                    command: self.command,
                    data: self.data.clone(),
                };
                self.reset();
                Ok(Some(frame))
            }
        }
    }

    fn process_delimiter(&mut self, byte: u8) -> Result<Option<RawFrame>, DecodeError> {
        let (frame_type, is_long) = match byte {
            b if b == delimiter::REQUEST_SHORT => (FrameType::Request, false),
            b if b == delimiter::REQUEST_LONG => (FrameType::Request, true),
            b if b == delimiter::RESPONSE_SHORT => (FrameType::Response, false),
            b if b == delimiter::RESPONSE_LONG => (FrameType::Response, true),
            b if b == delimiter::BURST_SHORT => (FrameType::Burst, false),
            b if b == delimiter::BURST_LONG => (FrameType::Burst, true),
            _ => {
                self.reset();
                return Err(DecodeError::InvalidDelimiter(byte));
            }
        };

        self.frame_type = Some(frame_type);
        self.is_long = is_long;
        self.addr_len = if is_long { 5 } else { 1 };
        self.addr_idx = 0;
        self.checksum = byte; // checksum starts at delimiter
        self.data.clear();
        self.state = State::Address(self.addr_len);

        Ok(None)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::encode_frame;
    use crate::types::{Address, FrameType, MasterRole};

    // Test vectors from test_vectors.rs
    const REQ_CMD0_SHORT_PRIMARY: &[u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x02,
        0x80,
        0x00,
        0x00,
        0x82,
    ];

    const REQ_CMD0_LONG_PRIMARY: &[u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x82,
        0x9A, 0x2B, 0x11, 0x22, 0x33,
        0x00,
        0x00,
        0x33,
    ];

    const RESP_CMD3_LONG: &[u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x86,
        0x9A, 0x2B, 0x11, 0x22, 0x33,
        0x03,
        0x1A,
        0x00, 0x00,
        0x41, 0x48, 0x00, 0x00,
        0x39, 0x42, 0x54, 0x80, 0x00,
        0x2D, 0x40, 0x20, 0x00, 0x00,
        0xFA, 0x7F, 0xC0, 0x00, 0x00,
        0x20, 0x41, 0xCA, 0x66, 0x66,
        0x2B,
    ];

    fn feed_all(decoder: &mut Decoder, bytes: &[u8]) -> Result<Option<RawFrame>, DecodeError> {
        let mut result = Ok(None);
        for &b in bytes {
            result = decoder.feed(b);
            if result.is_err() {
                return result;
            }
            if let Ok(Some(_)) = &result {
                return result;
            }
        }
        result
    }

    #[test]
    fn test_decode_short_request_test_vector() {
        let mut dec = Decoder::new();
        let frame = feed_all(&mut dec, REQ_CMD0_SHORT_PRIMARY).unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Request);
        assert_eq!(
            frame.address,
            Address::Short {
                master: MasterRole::Primary,
                burst: false,
                poll_address: 0
            }
        );
        assert_eq!(frame.command, 0);
        assert!(frame.data.is_empty());
    }

    #[test]
    fn test_decode_long_request_test_vector() {
        let mut dec = Decoder::new();
        let frame = feed_all(&mut dec, REQ_CMD0_LONG_PRIMARY).unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Request);
        assert_eq!(
            frame.address,
            Address::Long {
                master: MasterRole::Primary,
                burst: false,
                manufacturer_id: 0x1A,
                device_type: 0x2B,
                device_id: 0x112233
            }
        );
        assert_eq!(frame.command, 0);
        assert!(frame.data.is_empty());
    }

    #[test]
    fn test_roundtrip_short_request() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 64];
        let len = encode_frame(FrameType::Request, &address, 0, &[], 5, &mut buf).unwrap();

        let mut dec = Decoder::new();
        let frame = feed_all(&mut dec, &buf[..len]).unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Request);
        assert_eq!(frame.address, address);
        assert_eq!(frame.command, 0);
        assert!(frame.data.is_empty());
    }

    #[test]
    fn test_roundtrip_long_request_with_data() {
        let address = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x112233,
        };
        let payload = [0x01, 0x02, 0x03, 0x04];
        let mut buf = [0u8; 64];
        let len = encode_frame(FrameType::Request, &address, 42, &payload, 5, &mut buf).unwrap();

        let mut dec = Decoder::new();
        let frame = feed_all(&mut dec, &buf[..len]).unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Request);
        assert_eq!(frame.address, address);
        assert_eq!(frame.command, 42);
        assert_eq!(frame.data.as_slice(), &payload);
    }

    #[test]
    fn test_garbage_before_preamble_is_skipped() {
        let mut dec = Decoder::new();
        // Feed garbage bytes
        assert_eq!(dec.feed(0x00).unwrap(), None);
        assert_eq!(dec.feed(0x55).unwrap(), None);
        assert_eq!(dec.feed(0xAA).unwrap(), None);

        // Now feed the valid frame
        let frame = feed_all(&mut dec, REQ_CMD0_SHORT_PRIMARY).unwrap().unwrap();
        assert_eq!(frame.command, 0);
    }

    #[test]
    fn test_checksum_error_detected() {
        let mut bad_frame = REQ_CMD0_SHORT_PRIMARY.to_vec();
        // Corrupt the checksum byte (last byte)
        let last = bad_frame.len() - 1;
        bad_frame[last] ^= 0xFF; // flip all bits

        let mut dec = Decoder::new();
        let result = feed_all(&mut dec, &bad_frame);
        assert!(matches!(result, Err(DecodeError::ChecksumMismatch { .. })));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut dec = Decoder::new();
        // Feed a few preamble bytes
        dec.feed(0xFF).unwrap();
        dec.feed(0xFF).unwrap();
        dec.feed(0xFF).unwrap();

        dec.reset();

        // After reset, a new valid frame should decode correctly
        let frame = feed_all(&mut dec, REQ_CMD0_SHORT_PRIMARY).unwrap().unwrap();
        assert_eq!(frame.command, 0);
    }

    #[test]
    fn test_decode_cmd3_response_test_vector() {
        let mut dec = Decoder::new();
        let frame = feed_all(&mut dec, RESP_CMD3_LONG).unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Response);
        assert_eq!(frame.command, 3);
        // byte_count = 0x1A = 26
        assert_eq!(frame.data.len(), 26);
        // First data byte: status byte 0x00
        assert_eq!(frame.data[0], 0x00);
        assert_eq!(frame.data[1], 0x00);
    }

    #[test]
    fn test_insufficient_preamble_ignored() {
        // Only 4 preamble bytes (< MIN_PREAMBLE_COUNT=5), then a valid-looking delimiter
        let short_preamble: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0x02, 0x80, 0x00, 0x00, 0x82];
        let mut dec = Decoder::new();
        // None of these should produce a frame
        for &b in short_preamble {
            let _ = dec.feed(b);
        }
        // No frame should have been produced; decoder should be in Hunting or Preamble state
        // Feed a valid frame now and verify it decodes
        let frame = feed_all(&mut dec, REQ_CMD0_SHORT_PRIMARY).unwrap().unwrap();
        assert_eq!(frame.command, 0);
    }
}
