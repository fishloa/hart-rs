//! Command 9 — Read Device Variables with Status

use heapless::Vec;

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_DEVICE_VARS_WITH_STATUS;
use crate::error::{DecodeError, EncodeError};
use crate::units::UnitCode;

/// A single device variable entry in a Command 9 response.
///
/// Layout per variable (8 bytes):
///   [0]    device_var_code
///   [1]    classification
///   [2]    unit code
///   [3..6] value (f32 big-endian)
///   [7]    status
#[derive(Debug, Clone)]
pub struct DeviceVariable {
    pub device_var_code: u8,
    pub classification: u8,
    pub unit: UnitCode,
    pub value: f32,
    pub status: u8,
}

/// Command 9 request: up to 8 slot codes specifying which variables to read.
#[derive(Debug, Clone)]
pub struct ReadDeviceVarsRequest {
    pub slot_codes: Vec<u8, 8>,
}

/// Command 9 response: up to 8 device variables with status.
#[derive(Debug, Clone)]
pub struct ReadDeviceVarsResponse {
    pub variables: Vec<DeviceVariable, 8>,
}

impl CommandRequest for ReadDeviceVarsRequest {
    const COMMAND_NUMBER: u8 = READ_DEVICE_VARS_WITH_STATUS;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let len = self.slot_codes.len();
        if buf.len() < len {
            return Err(EncodeError::BufferTooSmall);
        }
        for (i, &code) in self.slot_codes.iter().enumerate() {
            buf[i] = code;
        }
        Ok(len)
    }
}

impl CommandResponse for ReadDeviceVarsResponse {
    const COMMAND_NUMBER: u8 = READ_DEVICE_VARS_WITH_STATUS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let n_vars = data.len() / 8;
        let n_vars = n_vars.min(8);
        let mut variables: Vec<DeviceVariable, 8> = Vec::new();

        for i in 0..n_vars {
            let base = i * 8;
            if base + 8 > data.len() {
                break;
            }
            let device_var_code = data[base];
            let classification = data[base + 1];
            let unit = UnitCode::from_u8(data[base + 2]);
            let value = f32::from_be_bytes([
                data[base + 3],
                data[base + 4],
                data[base + 5],
                data[base + 6],
            ]);
            let status = data[base + 7];
            let _ = variables.push(DeviceVariable {
                device_var_code,
                classification,
                unit,
                value,
                status,
            });
        }

        Ok(ReadDeviceVarsResponse { variables })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd9_command_number() {
        assert_eq!(ReadDeviceVarsRequest::COMMAND_NUMBER, 9);
        assert_eq!(ReadDeviceVarsResponse::COMMAND_NUMBER, 9);
    }

    #[test]
    fn test_cmd9_request_encode() {
        let mut slot_codes: Vec<u8, 8> = Vec::new();
        let _ = slot_codes.push(0x00);
        let _ = slot_codes.push(0x01);
        let req = ReadDeviceVarsRequest { slot_codes };
        let mut buf = [0u8; 8];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
        assert_eq!(buf[0], 0x00);
        assert_eq!(buf[1], 0x01);
    }

    #[test]
    fn test_cmd9_response_decode_one_variable() {
        // One variable: code=0x00, class=0x01, unit=45(Meters), value=2.5, status=0x00
        let value_bytes = 2.5f32.to_be_bytes();
        let mut data = [0u8; 8];
        data[0] = 0x00; // device_var_code
        data[1] = 0x01; // classification
        data[2] = 45; // unit: Meters
        data[3] = value_bytes[0];
        data[4] = value_bytes[1];
        data[5] = value_bytes[2];
        data[6] = value_bytes[3];
        data[7] = 0x00; // status

        let resp = ReadDeviceVarsResponse::decode_data(&data).unwrap();
        assert_eq!(resp.variables.len(), 1);
        let var = &resp.variables[0];
        assert_eq!(var.device_var_code, 0x00);
        assert_eq!(var.classification, 0x01);
        assert_eq!(var.unit, UnitCode::Meters);
        assert_eq!(var.value, 2.5f32);
        assert_eq!(var.status, 0x00);
    }

    #[test]
    fn test_cmd9_response_decode_empty() {
        let resp = ReadDeviceVarsResponse::decode_data(&[]).unwrap();
        assert_eq!(resp.variables.len(), 0);
    }

    #[test]
    fn test_cmd9_request_buffer_too_small() {
        let mut slot_codes: Vec<u8, 8> = Vec::new();
        for i in 0..4u8 {
            let _ = slot_codes.push(i);
        }
        let req = ReadDeviceVarsRequest { slot_codes };
        let mut buf = [0u8; 2]; // too small for 4 slots
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }
}
