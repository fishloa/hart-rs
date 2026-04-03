//! Core HART protocol types: addresses, frame types, and response status.

use crate::consts::address;
use crate::error::DecodeError;

/// Identifies the HART master that originated a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MasterRole {
    /// Primary master.
    Primary,
    /// Secondary master.
    Secondary,
}

/// The type of a HART frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    /// A command request from the master.
    Request,
    /// A response from the field device.
    Response,
    /// An unsolicited burst-mode message from the field device.
    Burst,
}

/// A HART address, either short (1 byte) or long (5 bytes).
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// Short-frame address (polling address, 1 byte on the wire).
    Short {
        /// The master role (primary or secondary).
        master: MasterRole,
        /// Whether the device is in burst mode.
        burst: bool,
        /// Polling address (0-15).
        poll_address: u8,
    },
    /// Long-frame (unique) address (5 bytes on the wire).
    Long {
        /// The master role (primary or secondary).
        master: MasterRole,
        /// Whether the device is in burst mode.
        burst: bool,
        /// Manufacturer identification code (6 bits).
        manufacturer_id: u8,
        /// Device type code.
        device_type: u8,
        /// 24-bit unique device identifier.
        device_id: u32,
    },
}

impl Address {
    /// Encodes the address into `buf`. Returns the number of bytes written
    /// (1 for short, 5 for long).
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        match self {
            Address::Short {
                master,
                burst,
                poll_address,
            } => {
                let mut byte = poll_address & address::SHORT_ADDRESS_MASK;
                if *master == MasterRole::Primary {
                    byte |= address::PRIMARY_MASTER_BIT;
                }
                if *burst {
                    byte |= address::BURST_MODE_BIT;
                }
                buf[0] = byte;
                1
            }
            Address::Long {
                master,
                burst,
                manufacturer_id,
                device_type,
                device_id,
            } => {
                // Byte 0: master bit | burst bit | manufacturer_id (6 bits)
                let mut b0 = manufacturer_id & address::MANUFACTURER_ID_MASK;
                if *master == MasterRole::Primary {
                    b0 |= address::PRIMARY_MASTER_BIT;
                }
                if *burst {
                    b0 |= address::BURST_MODE_BIT;
                }
                buf[0] = b0;
                buf[1] = *device_type;
                #[allow(clippy::cast_possible_truncation)]
                {
                    buf[2] = ((device_id >> 16) & 0xFF) as u8;
                    buf[3] = ((device_id >> 8) & 0xFF) as u8;
                    buf[4] = (device_id & 0xFF) as u8;
                }
                5
            }
        }
    }

    /// Decodes an address from `buf`. Returns `(Address, bytes_consumed)`.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeError::BufferTooShort`] if `buf` has fewer bytes than
    /// required for the address format.
    pub fn decode(buf: &[u8], is_long: bool) -> Result<(Self, usize), DecodeError> {
        if is_long {
            if buf.len() < 5 {
                return Err(DecodeError::BufferTooShort);
            }
            let b0 = buf[0];
            let master = if b0 & address::PRIMARY_MASTER_BIT != 0 {
                MasterRole::Primary
            } else {
                MasterRole::Secondary
            };
            let burst = b0 & address::BURST_MODE_BIT != 0;
            let manufacturer_id = b0 & address::MANUFACTURER_ID_MASK;
            let device_type = buf[1];
            let device_id =
                (u32::from(buf[2]) << 16) | (u32::from(buf[3]) << 8) | u32::from(buf[4]);
            Ok((
                Address::Long {
                    master,
                    burst,
                    manufacturer_id,
                    device_type,
                    device_id,
                },
                5,
            ))
        } else {
            if buf.is_empty() {
                return Err(DecodeError::BufferTooShort);
            }
            let b0 = buf[0];
            let master = if b0 & address::PRIMARY_MASTER_BIT != 0 {
                MasterRole::Primary
            } else {
                MasterRole::Secondary
            };
            let burst = b0 & address::BURST_MODE_BIT != 0;
            let poll_address = b0 & address::SHORT_ADDRESS_MASK;
            Ok((
                Address::Short {
                    master,
                    burst,
                    poll_address,
                },
                1,
            ))
        }
    }

    /// Returns true if this is a long address.
    #[must_use]
    pub fn is_long(&self) -> bool {
        matches!(self, Address::Long { .. })
    }
}

/// Response status bytes from a HART response frame.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseStatus {
    /// Communication status / error summary (byte 0).
    pub byte0: u8,
    /// Field device status flags (byte 1).
    pub byte1: u8,
}

impl ResponseStatus {
    /// Construct a `ResponseStatus` from the two raw status bytes.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        ResponseStatus {
            byte0: bytes[0],
            byte1: bytes[1],
        }
    }

    /// True if any communication error bit is set in byte 0.
    #[must_use]
    pub fn has_error(&self) -> bool {
        self.byte0 != 0
    }

    /// Byte 1 bit 7: field device malfunction.
    #[must_use]
    pub fn device_malfunction(&self) -> bool {
        self.byte1 & 0x80 != 0
    }

    /// Byte 1 bit 6: configuration changed.
    #[must_use]
    pub fn config_changed(&self) -> bool {
        self.byte1 & 0x40 != 0
    }

    /// Byte 1 bit 5: cold start.
    #[must_use]
    pub fn cold_start(&self) -> bool {
        self.byte1 & 0x20 != 0
    }

    /// Byte 1 bit 4: more status available.
    #[must_use]
    pub fn more_status_available(&self) -> bool {
        self.byte1 & 0x10 != 0
    }

    /// Byte 1 bit 3: primary variable out of limits.
    #[must_use]
    pub fn pv_out_of_limits(&self) -> bool {
        self.byte1 & 0x08 != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_address_roundtrip_primary_poll0() {
        // Primary master, no burst, poll_address=0 -> byte 0x80
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 1];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 1);
        assert_eq!(buf[0], 0x80);

        let (decoded, consumed) = Address::decode(&buf, false).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_short_address_roundtrip_various() {
        let addr = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 5,
        };
        let mut buf = [0u8; 1];
        addr.encode(&mut buf);
        let (decoded, _) = Address::decode(&buf, false).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_long_address_roundtrip() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x112233,
        };
        let mut buf = [0u8; 5];
        let len = addr.encode(&mut buf);
        assert_eq!(len, 5);

        let (decoded, consumed) = Address::decode(&buf, true).unwrap();
        assert_eq!(consumed, 5);
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_secondary_master_burst_address() {
        let addr = Address::Short {
            master: MasterRole::Secondary,
            burst: true,
            poll_address: 3,
        };
        let mut buf = [0u8; 1];
        addr.encode(&mut buf);
        // Secondary = no PRIMARY_MASTER_BIT (0x80), burst = BURST_MODE_BIT (0x40), poll=3
        assert_eq!(buf[0], 0x40 | 0x03);

        let (decoded, _) = Address::decode(&buf, false).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_long_address_is_long() {
        let short = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let long = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x01,
            device_type: 0x02,
            device_id: 0x000001,
        };
        assert!(!short.is_long());
        assert!(long.is_long());
    }

    #[test]
    fn test_response_status_flags() {
        // All clear
        let status = ResponseStatus::from_bytes([0x00, 0x00]);
        assert!(!status.has_error());
        assert!(!status.device_malfunction());
        assert!(!status.config_changed());
        assert!(!status.cold_start());
        assert!(!status.more_status_available());
        assert!(!status.pv_out_of_limits());

        // All flags set in byte 1
        let status = ResponseStatus::from_bytes([0x00, 0xF8]);
        assert!(!status.has_error());
        assert!(status.device_malfunction());
        assert!(status.config_changed());
        assert!(status.cold_start());
        assert!(status.more_status_available());
        assert!(status.pv_out_of_limits());

        // Communication error in byte 0
        let status = ResponseStatus::from_bytes([0x01, 0x00]);
        assert!(status.has_error());

        // Individual flags
        let status = ResponseStatus::from_bytes([0x00, 0x80]);
        assert!(status.device_malfunction());
        assert!(!status.config_changed());

        let status = ResponseStatus::from_bytes([0x00, 0x40]);
        assert!(!status.device_malfunction());
        assert!(status.config_changed());

        let status = ResponseStatus::from_bytes([0x00, 0x20]);
        assert!(status.cold_start());

        let status = ResponseStatus::from_bytes([0x00, 0x10]);
        assert!(status.more_status_available());

        let status = ResponseStatus::from_bytes([0x00, 0x08]);
        assert!(status.pv_out_of_limits());
    }

    #[test]
    fn test_decode_buffer_too_short_short() {
        let result = Address::decode(&[], false);
        assert_eq!(result, Err(DecodeError::BufferTooShort));
    }

    #[test]
    fn test_decode_buffer_too_short_long() {
        let result = Address::decode(&[0x00, 0x01, 0x02], true);
        assert_eq!(result, Err(DecodeError::BufferTooShort));
    }

    #[test]
    fn test_short_address_all_poll_addresses() {
        for poll in 0..=15u8 {
            let addr = Address::Short {
                master: MasterRole::Primary,
                burst: false,
                poll_address: poll,
            };
            let mut buf = [0u8; 1];
            addr.encode(&mut buf);
            let (decoded, consumed) = Address::decode(&buf, false).unwrap();
            assert_eq!(consumed, 1);
            assert_eq!(decoded, addr, "roundtrip failed for poll_address={}", poll);
        }
    }

    #[test]
    fn test_long_address_manufacturer_id_zero() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0,
            device_type: 0,
            device_id: 0,
        };
        let mut buf = [0u8; 5];
        addr.encode(&mut buf);
        let (decoded, _) = Address::decode(&buf, true).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_long_address_manufacturer_id_max() {
        let addr = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x3F, // max 6-bit value
            device_type: 0xFF,
            device_id: 0x00FF_FFFF,
        };
        let mut buf = [0u8; 5];
        addr.encode(&mut buf);
        let (decoded, _) = Address::decode(&buf, true).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_long_address_device_id_zero() {
        let addr = Address::Long {
            master: MasterRole::Secondary,
            burst: true,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0,
        };
        let mut buf = [0u8; 5];
        addr.encode(&mut buf);
        let (decoded, _) = Address::decode(&buf, true).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn test_response_status_all_byte0_errors() {
        // All bits set in byte 0 = communication error
        let status = ResponseStatus::from_bytes([0xFF, 0x00]);
        assert!(status.has_error());
    }

    #[test]
    fn test_decode_buffer_too_short_long_4_bytes() {
        let result = Address::decode(&[0x00, 0x01, 0x02, 0x03], true);
        assert_eq!(result, Err(DecodeError::BufferTooShort));
    }

    #[test]
    fn test_long_address_exactly_5_bytes() {
        let result = Address::decode(&[0x80, 0x00, 0x00, 0x00, 0x00], true);
        assert!(result.is_ok());
        let (addr, consumed) = result.unwrap();
        assert_eq!(consumed, 5);
        assert!(addr.is_long());
    }
}
