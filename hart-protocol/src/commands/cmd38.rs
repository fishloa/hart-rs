/// Command 38 — Reset Configuration Changed Flag

use crate::consts::commands::RESET_CONFIG_CHANGED;
use crate::error::{DecodeError, EncodeError};
use super::{CommandRequest, CommandResponse};

/// Command 38 request: configuration change counter (u16), encoded as 2 bytes big-endian.
#[derive(Debug, Clone)]
pub struct Cmd38Request {
    pub configuration_change_counter: u16,
}

/// Command 38 response: echoes the configuration change counter.
///
/// Layout (2 bytes):
///   [0..1] configuration_change_counter (big-endian u16)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd38Response {
    pub configuration_change_counter: u16,
}

impl CommandRequest for Cmd38Request {
    const COMMAND_NUMBER: u8 = RESET_CONFIG_CHANGED;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 2 {
            return Err(EncodeError::BufferTooSmall);
        }
        let c = self.configuration_change_counter;
        buf[0] = (c >> 8) as u8;
        buf[1] = (c & 0xFF) as u8;
        Ok(2)
    }
}

impl CommandResponse for Cmd38Response {
    const COMMAND_NUMBER: u8 = RESET_CONFIG_CHANGED;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::BufferTooShort);
        }
        let configuration_change_counter = ((data[0] as u16) << 8) | (data[1] as u16);
        Ok(Cmd38Response { configuration_change_counter })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd38_command_number() {
        assert_eq!(Cmd38Request::COMMAND_NUMBER, 38);
        assert_eq!(Cmd38Response::COMMAND_NUMBER, 38);
    }

    #[test]
    fn test_cmd38_roundtrip() {
        let req = Cmd38Request { configuration_change_counter: 0x1234 };
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
        assert_eq!(&buf[..2], &[0x12, 0x34]);

        let resp = Cmd38Response::decode_data(&buf[..2]).unwrap();
        assert_eq!(resp.configuration_change_counter, 0x1234);
    }

    #[test]
    fn test_cmd38_request_buffer_too_small() {
        let req = Cmd38Request { configuration_change_counter: 0 };
        let mut buf = [0u8; 1]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd38_response_too_short() {
        let data = [0x12u8]; // needs 2
        assert_eq!(
            Cmd38Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd38_counter_zero() {
        let req = Cmd38Request { configuration_change_counter: 0 };
        let mut buf = [0u8; 2];
        let len = req.encode_data(&mut buf).unwrap();
        let resp = Cmd38Response::decode_data(&buf[..len]).unwrap();
        assert_eq!(resp.configuration_change_counter, 0);
    }
}
