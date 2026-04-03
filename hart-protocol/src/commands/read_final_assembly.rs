//! Command 16 — Read Final Assembly Number

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_FINAL_ASSEMBLY_NUMBER;
use crate::error::{DecodeError, EncodeError};

/// Command 16 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadFinalAssemblyRequest;

/// Command 16 response: 24-bit final assembly number.
///
/// Layout (3 bytes):
///   0..2: `final_assembly_number` (24-bit big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct ReadFinalAssemblyResponse {
    /// 24-bit final assembly number.
    pub final_assembly_number: u32,
}

impl CommandRequest for ReadFinalAssemblyRequest {
    const COMMAND_NUMBER: u8 = READ_FINAL_ASSEMBLY_NUMBER;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadFinalAssemblyResponse {
    const COMMAND_NUMBER: u8 = READ_FINAL_ASSEMBLY_NUMBER;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 3 {
            return Err(DecodeError::BufferTooShort);
        }
        let final_assembly_number = super::decode_u24_be(&data[0..3]);
        Ok(ReadFinalAssemblyResponse {
            final_assembly_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd16_command_number() {
        assert_eq!(ReadFinalAssemblyRequest::COMMAND_NUMBER, 16);
        assert_eq!(ReadFinalAssemblyResponse::COMMAND_NUMBER, 16);
    }

    #[test]
    fn test_cmd16_request_encodes_no_data() {
        let req = ReadFinalAssemblyRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd16_response_decode() {
        let data = [0x01u8, 0x02, 0x03];
        let resp = ReadFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0x010203);
    }

    #[test]
    fn test_cmd16_response_too_short() {
        let data = [0x01u8, 0x02]; // needs 3
        assert_eq!(
            ReadFinalAssemblyResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd16_response_max_value() {
        let data = [0xFFu8, 0xFF, 0xFF];
        let resp = ReadFinalAssemblyResponse::decode_data(&data).unwrap();
        assert_eq!(resp.final_assembly_number, 0x00FF_FFFF);
    }
}
