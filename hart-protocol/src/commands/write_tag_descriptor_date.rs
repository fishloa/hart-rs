//! Command 18 — Write Tag, Descriptor, and Date

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::WRITE_TAG_DESCRIPTOR_DATE;
use crate::error::{DecodeError, EncodeError};
use crate::packed_string::{decode_packed, encode_packed};

/// Command 18 request: tag (8 chars), descriptor (16 chars), and date.
#[derive(Debug, Clone)]
pub struct WriteTagDescriptorDateRequest {
    /// 8-character ASCII tag.
    pub tag: [u8; 8],
    /// 16-character ASCII descriptor.
    pub descriptor: [u8; 16],
    /// Day of month.
    pub day: u8,
    /// Month.
    pub month: u8,
    /// Year (years since 1900).
    pub year: u8,
}

/// Command 18 response: echoes tag, descriptor, and date (same as Command 13 response).
///
/// Layout (21 bytes):
///   0..5:   tag: 6 packed bytes → 8 ASCII chars
///   6..17:  descriptor: 12 packed bytes → 16 ASCII chars
///   18:     day
///   19:     month
///   20:     year
#[derive(Debug, Clone, PartialEq)]
pub struct WriteTagDescriptorDateResponse {
    /// Echoed 8-character ASCII tag.
    pub tag: [u8; 8],
    /// Echoed 16-character ASCII descriptor.
    pub descriptor: [u8; 16],
    /// Echoed day of month.
    pub day: u8,
    /// Echoed month.
    pub month: u8,
    /// Echoed year (years since 1900).
    pub year: u8,
}

impl CommandRequest for WriteTagDescriptorDateRequest {
    const COMMAND_NUMBER: u8 = WRITE_TAG_DESCRIPTOR_DATE;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 21 {
            return Err(EncodeError::BufferTooSmall);
        }
        // Encode 8-char tag into 6 bytes
        encode_packed(&self.tag, &mut buf[0..6]);
        // Encode 16-char descriptor into 12 bytes
        encode_packed(&self.descriptor, &mut buf[6..18]);
        // Date
        buf[18] = self.day;
        buf[19] = self.month;
        buf[20] = self.year;
        Ok(21)
    }
}

impl CommandResponse for WriteTagDescriptorDateResponse {
    const COMMAND_NUMBER: u8 = WRITE_TAG_DESCRIPTOR_DATE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 21 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut tag = [b' '; 8];
        decode_packed(&data[0..6], &mut tag);

        let mut descriptor = [b' '; 16];
        decode_packed(&data[6..18], &mut descriptor);

        let day = data[18];
        let month = data[19];
        let year = data[20];

        Ok(WriteTagDescriptorDateResponse {
            tag,
            descriptor,
            day,
            month,
            year,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd18_command_number() {
        assert_eq!(WriteTagDescriptorDateRequest::COMMAND_NUMBER, 18);
        assert_eq!(WriteTagDescriptorDateResponse::COMMAND_NUMBER, 18);
    }

    #[test]
    fn test_cmd18_roundtrip() {
        let mut tag = [b' '; 8];
        tag[..8].copy_from_slice(b"SENSOR01");
        let mut desc = [b' '; 16];
        desc[..10].copy_from_slice(b"MY SENSOR ");

        let req = WriteTagDescriptorDateRequest {
            tag,
            descriptor: desc,
            day: 10,
            month: 3,
            year: 124, // 2024
        };

        let mut buf = [0u8; 21];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 21);

        let resp = WriteTagDescriptorDateResponse::decode_data(&buf[..len]).unwrap();
        assert_eq!(&resp.tag, b"SENSOR01");
        assert_eq!(&resp.descriptor[..10], b"MY SENSOR ");
        assert_eq!(resp.day, 10);
        assert_eq!(resp.month, 3);
        assert_eq!(resp.year, 124);
    }

    #[test]
    fn test_cmd18_request_buffer_too_small() {
        let req = WriteTagDescriptorDateRequest {
            tag: [b' '; 8],
            descriptor: [b' '; 16],
            day: 1,
            month: 1,
            year: 0,
        };
        let mut buf = [0u8; 20]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd18_response_too_short() {
        let data = [0u8; 20]; // needs 21
        assert_eq!(
            WriteTagDescriptorDateResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
