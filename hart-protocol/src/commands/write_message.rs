//! Command 17 — Write Message

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::WRITE_MESSAGE;
use crate::error::{DecodeError, EncodeError};
use crate::packed_string::{decode_packed, encode_packed};

/// Command 17 request: 32-character message, encoded as 24 packed bytes on the wire.
#[derive(Debug, Clone)]
pub struct WriteMessageRequest {
    /// 32-character ASCII message (padded with spaces if shorter).
    pub message: [u8; 32],
}

/// Command 17 response: echoes the message, decoded from 24 packed bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteMessageResponse {
    /// 32-character ASCII message echoed from the device.
    pub message: [u8; 32],
}

impl CommandRequest for WriteMessageRequest {
    const COMMAND_NUMBER: u8 = WRITE_MESSAGE;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 24 {
            return Err(EncodeError::BufferTooSmall);
        }
        encode_packed(&self.message, &mut buf[..24]);
        Ok(24)
    }
}

impl CommandResponse for WriteMessageResponse {
    const COMMAND_NUMBER: u8 = WRITE_MESSAGE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 24 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut message = [b' '; 32];
        decode_packed(&data[..24], &mut message);
        Ok(WriteMessageResponse { message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd17_command_number() {
        assert_eq!(WriteMessageRequest::COMMAND_NUMBER, 17);
        assert_eq!(WriteMessageResponse::COMMAND_NUMBER, 17);
    }

    #[test]
    fn test_cmd17_request_encode_and_response_decode_roundtrip() {
        let mut msg = [b' '; 32];
        msg[..11].copy_from_slice(b"HELLO WORLD");
        let req = WriteMessageRequest { message: msg };

        let mut buf = [0u8; 24];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 24);

        let resp = WriteMessageResponse::decode_data(&buf[..len]).unwrap();
        assert_eq!(&resp.message[..11], b"HELLO WORLD");
        assert_eq!(&resp.message[11..], &[b' '; 21]);
    }

    #[test]
    fn test_cmd17_request_buffer_too_small() {
        let req = WriteMessageRequest {
            message: [b' '; 32],
        };
        let mut buf = [0u8; 20]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd17_response_too_short() {
        let data = [0u8; 23]; // needs 24
        assert_eq!(
            WriteMessageResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
