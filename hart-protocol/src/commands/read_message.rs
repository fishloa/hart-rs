//! Command 12 — Read Message

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_MESSAGE;
use crate::error::{DecodeError, EncodeError};
use crate::packed_string::decode_packed;

/// Command 12 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadMessageRequest;

/// Command 12 response: 32-character message decoded from 24 packed bytes.
///
/// Layout (24 packed bytes → 32 ASCII chars):
///   The message field is stored as 24 bytes of 6-bit packed ASCII,
///   which decodes to 32 characters.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadMessageResponse {
    pub message: [u8; 32],
}

impl CommandRequest for ReadMessageRequest {
    const COMMAND_NUMBER: u8 = READ_MESSAGE;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadMessageResponse {
    const COMMAND_NUMBER: u8 = READ_MESSAGE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 24 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut message = [b' '; 32];
        decode_packed(&data[..24], &mut message);
        Ok(ReadMessageResponse { message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed_string::encode_packed;

    #[test]
    fn test_cmd12_command_number() {
        assert_eq!(ReadMessageRequest::COMMAND_NUMBER, 12);
        assert_eq!(ReadMessageResponse::COMMAND_NUMBER, 12);
    }

    #[test]
    fn test_cmd12_request_encodes_no_data() {
        let req = ReadMessageRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd12_response_decode() {
        // Encode "HELLO HART DEVICE MESSAGE       " (32 chars) into 24 packed bytes
        let msg_str = b"HELLO HART DEVICE MESSAGE       ";
        let mut packed = [0u8; 24];
        encode_packed(msg_str, &mut packed);

        let resp = ReadMessageResponse::decode_data(&packed).unwrap();
        assert_eq!(&resp.message, msg_str);
    }

    #[test]
    fn test_cmd12_response_too_short() {
        let data = [0u8; 23]; // needs 24
        assert_eq!(
            ReadMessageResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
