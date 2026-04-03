//! Command 20 — Read Long Tag

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_LONG_TAG;
use crate::error::{DecodeError, EncodeError};

/// Command 20 request: no data payload.
#[derive(Debug, Clone)]
pub struct Cmd20Request;

/// Command 20 response: 32-byte plain ASCII long tag.
///
/// Layout (32 bytes):
///   [0..31] long_tag — plain ASCII (NOT packed), padded with spaces.
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd20Response {
    pub long_tag: [u8; 32],
}

impl CommandRequest for Cmd20Request {
    const COMMAND_NUMBER: u8 = READ_LONG_TAG;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for Cmd20Response {
    const COMMAND_NUMBER: u8 = READ_LONG_TAG;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 32 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut long_tag = [b' '; 32];
        long_tag.copy_from_slice(&data[..32]);
        Ok(Cmd20Response { long_tag })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd20_command_number() {
        assert_eq!(Cmd20Request::COMMAND_NUMBER, 20);
        assert_eq!(Cmd20Response::COMMAND_NUMBER, 20);
    }

    #[test]
    fn test_cmd20_request_encodes_no_data() {
        let req = Cmd20Request;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd20_response_decode() {
        let mut data = [b' '; 32];
        data[..11].copy_from_slice(b"MY LONG TAG");
        let resp = Cmd20Response::decode_data(&data).unwrap();
        assert_eq!(&resp.long_tag[..11], b"MY LONG TAG");
        assert_eq!(&resp.long_tag[11..], &[b' '; 21]);
    }

    #[test]
    fn test_cmd20_response_too_short() {
        let data = [0u8; 31]; // needs 32
        assert_eq!(
            Cmd20Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd20_response_full_32_bytes() {
        let data = [b'A'; 32];
        let resp = Cmd20Response::decode_data(&data).unwrap();
        assert_eq!(resp.long_tag, [b'A'; 32]);
    }
}
