//! Command 22 — Write Long Tag

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::WRITE_LONG_TAG;
use crate::error::{DecodeError, EncodeError};

/// Command 22 request: 32-byte plain ASCII long tag.
#[derive(Debug, Clone)]
pub struct WriteLongTagRequest {
    /// 32-character plain ASCII long tag (padded with spaces).
    pub long_tag: [u8; 32],
}

/// Command 22 response: echoes the 32-byte plain ASCII long tag.
///
/// Layout (32 bytes):
///   [0..31] long_tag — plain ASCII (NOT packed).
#[derive(Debug, Clone, PartialEq)]
pub struct WriteLongTagResponse {
    pub long_tag: [u8; 32],
}

impl CommandRequest for WriteLongTagRequest {
    const COMMAND_NUMBER: u8 = WRITE_LONG_TAG;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 32 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[..32].copy_from_slice(&self.long_tag);
        Ok(32)
    }
}

impl CommandResponse for WriteLongTagResponse {
    const COMMAND_NUMBER: u8 = WRITE_LONG_TAG;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 32 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut long_tag = [b' '; 32];
        long_tag.copy_from_slice(&data[..32]);
        Ok(WriteLongTagResponse { long_tag })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd22_command_number() {
        assert_eq!(WriteLongTagRequest::COMMAND_NUMBER, 22);
        assert_eq!(WriteLongTagResponse::COMMAND_NUMBER, 22);
    }

    #[test]
    fn test_cmd22_roundtrip() {
        let mut long_tag = [b' '; 32];
        long_tag[..15].copy_from_slice(b"MY LONG TAG 123");
        let req = WriteLongTagRequest { long_tag };

        let mut buf = [0u8; 32];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 32);

        let resp = WriteLongTagResponse::decode_data(&buf[..len]).unwrap();
        assert_eq!(&resp.long_tag[..15], b"MY LONG TAG 123");
        assert_eq!(&resp.long_tag[15..], &[b' '; 17]);
    }

    #[test]
    fn test_cmd22_request_buffer_too_small() {
        let req = WriteLongTagRequest {
            long_tag: [b' '; 32],
        };
        let mut buf = [0u8; 31]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd22_response_too_short() {
        let data = [0u8; 31]; // needs 32
        assert_eq!(
            WriteLongTagResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
