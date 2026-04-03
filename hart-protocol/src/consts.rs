//! HART protocol constants for frame encoding, UART configuration, and
//! command numbers.

/// HART preamble byte (all ones).
pub const PREAMBLE_BYTE: u8 = 0xFF;
/// Minimum number of preamble bytes required in a HART frame.
pub const MIN_PREAMBLE_COUNT: u8 = 5;
/// Maximum number of preamble bytes allowed in a HART frame.
pub const MAX_PREAMBLE_COUNT: u8 = 20;
/// Default number of preamble bytes used when encoding frames.
pub const DEFAULT_PREAMBLE_COUNT: u8 = 10;
/// Maximum data payload length in a HART frame (255 bytes).
pub const MAX_DATA_LENGTH: usize = 255;
/// Maximum total HART frame length including preamble, header, data, and checksum.
pub const MAX_FRAME_LENGTH: usize = 284;

/// HART UART baud rate (1200 baud, per the HART Physical Layer spec).
pub const BAUD_RATE: u32 = 1200;
/// Default response timeout in milliseconds.
pub const RESPONSE_TIMEOUT_MS: u32 = 500;
/// Time in milliseconds to hold RTS asserted before transmitting.
pub const RTS_HOLD_TIME_MS: u32 = 5;

/// HART frame delimiter byte constants.
pub mod delimiter {
    /// Short-frame request delimiter.
    pub const REQUEST_SHORT: u8 = 0x02;
    /// Long-frame request delimiter.
    pub const REQUEST_LONG: u8 = 0x82;
    /// Short-frame response delimiter.
    pub const RESPONSE_SHORT: u8 = 0x06;
    /// Long-frame response delimiter.
    pub const RESPONSE_LONG: u8 = 0x86;
    /// Short-frame burst delimiter.
    pub const BURST_SHORT: u8 = 0x01;
    /// Long-frame burst delimiter.
    pub const BURST_LONG: u8 = 0x81;
    /// Bit mask indicating a long-address frame.
    pub const LONG_ADDRESS_BIT: u8 = 0x80;
}

/// HART address byte bit-field constants.
pub mod address {
    /// Bit set when the primary master originated the frame.
    pub const PRIMARY_MASTER_BIT: u8 = 0x80;
    /// Bit set when the device is in burst mode.
    pub const BURST_MODE_BIT: u8 = 0x40;
    /// Mask for the 4-bit short polling address.
    pub const SHORT_ADDRESS_MASK: u8 = 0x0F;
    /// Mask for the 6-bit manufacturer ID in long addresses.
    pub const MANUFACTURER_ID_MASK: u8 = 0x3F;
}

/// HART command number constants.
pub mod commands {
    /// Command 0 — Read Unique Identifier (device ID).
    pub const READ_DEVICE_ID: u8 = 0;
    /// Command 1 — Read Primary Variable.
    pub const READ_PRIMARY_VARIABLE: u8 = 1;
    /// Command 2 — Read Loop Current and Percent of Range.
    pub const READ_LOOP_CURRENT_PERCENT: u8 = 2;
    /// Command 3 — Read Dynamic Variables and Loop Current.
    pub const READ_DYNAMIC_VARS: u8 = 3;
    /// Command 6 — Write Polling Address.
    pub const WRITE_POLLING_ADDRESS: u8 = 6;
    /// Command 7 — Read Loop Configuration.
    pub const READ_LOOP_CONFIG: u8 = 7;
    /// Command 8 — Read Dynamic Variable Classifications.
    pub const READ_DYNAMIC_VAR_CLASS: u8 = 8;
    /// Command 9 — Read Device Variables with Status.
    pub const READ_DEVICE_VARS_WITH_STATUS: u8 = 9;
    /// Command 11 — Read Unique Identifier by Tag.
    pub const READ_UNIQUE_ID_BY_TAG: u8 = 11;
    /// Command 12 — Read Message.
    pub const READ_MESSAGE: u8 = 12;
    /// Command 13 — Read Tag, Descriptor, and Date.
    pub const READ_TAG_DESCRIPTOR_DATE: u8 = 13;
    /// Command 14 — Read Primary Variable Transducer Information.
    pub const READ_PV_TRANSDUCER_INFO: u8 = 14;
    /// Command 15 — Read Device Information.
    pub const READ_DEVICE_INFO: u8 = 15;
    /// Command 16 — Read Final Assembly Number.
    pub const READ_FINAL_ASSEMBLY_NUMBER: u8 = 16;
    /// Command 17 — Write Message.
    pub const WRITE_MESSAGE: u8 = 17;
    /// Command 18 — Write Tag, Descriptor, and Date.
    pub const WRITE_TAG_DESCRIPTOR_DATE: u8 = 18;
    /// Command 19 — Write Final Assembly Number.
    pub const WRITE_FINAL_ASSEMBLY_NUMBER: u8 = 19;
    /// Command 20 — Read Long Tag.
    pub const READ_LONG_TAG: u8 = 20;
    /// Command 21 — Read Unique Identifier by Long Tag (defined for future use).
    pub const READ_UNIQUE_ID_BY_LONG_TAG: u8 = 21;
    /// Command 22 — Write Long Tag.
    pub const WRITE_LONG_TAG: u8 = 22;
    /// Command 38 — Reset Configuration Changed Flag.
    pub const RESET_CONFIG_CHANGED: u8 = 38;
    /// Command 48 — Read Additional Device Status.
    pub const READ_ADDITIONAL_STATUS: u8 = 48;
}
