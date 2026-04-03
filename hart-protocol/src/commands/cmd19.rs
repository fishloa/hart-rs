//! Command 19 — Write Final Assembly Number

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::WRITE_FINAL_ASSEMBLY_NUMBER;
use crate::error::{DecodeError, EncodeError};

/// Command 19 request: 24-bit final assembly number encoded as 3 bytes big-endian.
#[derive(Debug, Clone)]
pub struct Cmd19Request {
    pub final_assembly_number: u32,
}

/// Command 19 response: echoes the 24-bit final assembly number.
///
/// Layout (3 bytes):
///   [0..2] final_assembly_number (24-bit big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd19Response {
    pub final_assembly_number: u32,
}

impl CommandRequest for Cmd19Request {
    const COMMAND_NUMBER: u8 = WRITE_FINAL_ASSEMBLY_NUMBER;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 3 {
            return Err(EncodeError::BufferTooSmall);
        }
        super::encode_u24_be(self.final_assembly_number, &mut buf[0..3]);
        Ok(3)
    }
}

impl CommandResponse for Cmd19Response {
    const COMMAND_NUMBER: u8 = WRITE_FINAL_ASSEMBLY_NUMBER;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 3 {
            return Err(DecodeError::BufferTooShort);
        }
        let final_assembly_number = super::decode_u24_be(&data[0..3]);
        Ok(Cmd19Response {
            final_assembly_number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd19_command_number() {
        assert_eq!(Cmd19Request::COMMAND_NUMBER, 19);
        assert_eq!(Cmd19Response::COMMAND_NUMBER, 19);
    }

    #[test]
    fn test_cmd19_roundtrip() {
        let req = Cmd19Request {
            final_assembly_number: 0xABCDEF,
        };
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 3);
        assert_eq!(&buf[..3], &[0xAB, 0xCD, 0xEF]);

        let resp = Cmd19Response::decode_data(&buf[..3]).unwrap();
        assert_eq!(resp.final_assembly_number, 0xABCDEF);
    }

    #[test]
    fn test_cmd19_request_buffer_too_small() {
        let req = Cmd19Request {
            final_assembly_number: 1,
        };
        let mut buf = [0u8; 2]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd19_response_too_short() {
        let data = [0x01u8, 0x02]; // needs 3
        assert_eq!(
            Cmd19Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd19_encode_zero() {
        let req = Cmd19Request {
            final_assembly_number: 0,
        };
        let mut buf = [0u8; 3];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 3);
        assert_eq!(&buf[..3], &[0x00, 0x00, 0x00]);
    }
}
