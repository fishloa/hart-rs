//! Command 48 — Read Additional Device Status

use heapless::Vec;

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_ADDITIONAL_STATUS;
use crate::error::{DecodeError, EncodeError};

/// Maximum number of additional status bytes for Command 48.
const CMD48_MAX_STATUS_BYTES: usize = 25;

/// Command 48 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadAdditionalStatusRequest;

/// Command 48 response: up to 25 device-specific status bytes.
#[derive(Debug, Clone)]
pub struct ReadAdditionalStatusResponse {
    pub data: Vec<u8, CMD48_MAX_STATUS_BYTES>,
}

impl CommandRequest for ReadAdditionalStatusRequest {
    const COMMAND_NUMBER: u8 = READ_ADDITIONAL_STATUS;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadAdditionalStatusResponse {
    const COMMAND_NUMBER: u8 = READ_ADDITIONAL_STATUS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let mut vec: Vec<u8, CMD48_MAX_STATUS_BYTES> = Vec::new();
        let copy_len = data.len().min(CMD48_MAX_STATUS_BYTES);
        for &b in &data[..copy_len] {
            // Vec capacity is CMD48_MAX_STATUS_BYTES = 25, copy_len <= 25, so this never fails
            let _ = vec.push(b);
        }
        Ok(ReadAdditionalStatusResponse { data: vec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd48_request_encodes_no_data() {
        let req = ReadAdditionalStatusRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd48_command_number() {
        assert_eq!(ReadAdditionalStatusRequest::COMMAND_NUMBER, 48);
        assert_eq!(ReadAdditionalStatusResponse::COMMAND_NUMBER, 48);
    }

    #[test]
    fn test_cmd48_response_decode_5_bytes() {
        // From RESP_CMD48_LONG test vector (after stripping 2 status bytes):
        // 5 device-specific status bytes: 0x00, 0x01, 0x02, 0x03, 0x04
        let data = [0x00u8, 0x01, 0x02, 0x03, 0x04];
        let resp = ReadAdditionalStatusResponse::decode_data(&data).unwrap();
        assert_eq!(resp.data.as_slice(), &[0x00, 0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_cmd48_response_decode_empty() {
        let resp = ReadAdditionalStatusResponse::decode_data(&[]).unwrap();
        assert!(resp.data.is_empty());
    }

    #[test]
    fn test_cmd48_response_decode_truncates_at_25() {
        // More than 25 bytes: should only copy first 25
        let data = [0xAAu8; 30];
        let resp = ReadAdditionalStatusResponse::decode_data(&data).unwrap();
        assert_eq!(resp.data.len(), 25);
        assert!(resp.data.iter().all(|&b| b == 0xAA));
    }
}
