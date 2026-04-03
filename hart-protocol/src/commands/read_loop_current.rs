//! Command 2 — Read Loop Current and Percent of Range

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_LOOP_CURRENT_PERCENT;
use crate::error::{DecodeError, EncodeError};

/// Command 2 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadLoopCurrentRequest;

/// Command 2 response: loop current (mA) and percent of range.
///
/// Layout (8 bytes):
///   [0..3] loop current in mA (f32 big-endian)
///   [4..7] percent of range (f32 big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct ReadLoopCurrentResponse {
    pub current_ma: f32,
    pub percent_of_range: f32,
}

impl CommandRequest for ReadLoopCurrentRequest {
    const COMMAND_NUMBER: u8 = READ_LOOP_CURRENT_PERCENT;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadLoopCurrentResponse {
    const COMMAND_NUMBER: u8 = READ_LOOP_CURRENT_PERCENT;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 8 {
            return Err(DecodeError::BufferTooShort);
        }
        let current_ma = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let percent_of_range = f32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Ok(ReadLoopCurrentResponse {
            current_ma,
            percent_of_range,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd2_request_encodes_no_data() {
        let req = ReadLoopCurrentRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd2_response_decode() {
        // From RESP_CMD2_LONG test vector (after stripping 2 status bytes):
        // current=12.5 mA, percent=53.125%
        let data = [
            0x41, 0x48, 0x00, 0x00, // 12.5
            0x42, 0x54, 0x80, 0x00, // 53.125
        ];
        let resp = ReadLoopCurrentResponse::decode_data(&data).unwrap();
        assert_eq!(resp.current_ma, 12.5f32);
        assert_eq!(resp.percent_of_range, 53.125f32);
    }

    #[test]
    fn test_cmd2_response_too_short() {
        let data = [0x41, 0x48, 0x00]; // only 3 bytes, needs 8
        assert_eq!(
            ReadLoopCurrentResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd2_command_number() {
        assert_eq!(ReadLoopCurrentRequest::COMMAND_NUMBER, 2);
        assert_eq!(ReadLoopCurrentResponse::COMMAND_NUMBER, 2);
    }
}
