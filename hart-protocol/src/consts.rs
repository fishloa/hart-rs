// --- Frame constants ---
pub const PREAMBLE_BYTE: u8 = 0xFF;
pub const MIN_PREAMBLE_COUNT: u8 = 5;
pub const MAX_PREAMBLE_COUNT: u8 = 20;
pub const DEFAULT_PREAMBLE_COUNT: u8 = 10;
pub const MAX_DATA_LENGTH: usize = 255;
pub const MAX_FRAME_LENGTH: usize = 284;

// --- UART configuration ---
pub const BAUD_RATE: u32 = 1200;
pub const RESPONSE_TIMEOUT_MS: u32 = 500;
pub const RTS_HOLD_TIME_MS: u32 = 5;

pub mod delimiter {
    pub const REQUEST_SHORT: u8 = 0x02;
    pub const REQUEST_LONG: u8 = 0x82;
    pub const RESPONSE_SHORT: u8 = 0x06;
    pub const RESPONSE_LONG: u8 = 0x86;
    pub const BURST_SHORT: u8 = 0x01;
    pub const BURST_LONG: u8 = 0x81;
    pub const LONG_ADDRESS_BIT: u8 = 0x80;
}

pub mod address {
    pub const PRIMARY_MASTER_BIT: u8 = 0x80;
    pub const BURST_MODE_BIT: u8 = 0x40;
    pub const SHORT_ADDRESS_MASK: u8 = 0x0F;
    pub const MANUFACTURER_ID_MASK: u8 = 0x3F;
}

pub mod commands {
    pub const READ_DEVICE_ID: u8 = 0;
    pub const READ_PRIMARY_VARIABLE: u8 = 1;
    pub const READ_LOOP_CURRENT_PERCENT: u8 = 2;
    pub const READ_DYNAMIC_VARS: u8 = 3;
    pub const WRITE_POLLING_ADDRESS: u8 = 6;
    pub const READ_LOOP_CONFIG: u8 = 7;
    pub const READ_DYNAMIC_VAR_CLASS: u8 = 8;
    pub const READ_DEVICE_VARS_WITH_STATUS: u8 = 9;
    pub const READ_UNIQUE_ID_BY_TAG: u8 = 11;
    pub const READ_MESSAGE: u8 = 12;
    pub const READ_TAG_DESCRIPTOR_DATE: u8 = 13;
    pub const READ_PV_TRANSDUCER_INFO: u8 = 14;
    pub const READ_DEVICE_INFO: u8 = 15;
    pub const READ_FINAL_ASSEMBLY_NUMBER: u8 = 16;
    pub const WRITE_MESSAGE: u8 = 17;
    pub const WRITE_TAG_DESCRIPTOR_DATE: u8 = 18;
    pub const WRITE_FINAL_ASSEMBLY_NUMBER: u8 = 19;
    pub const READ_LONG_TAG: u8 = 20;
    pub const READ_UNIQUE_ID_BY_LONG_TAG: u8 = 21;
    pub const WRITE_LONG_TAG: u8 = 22;
    pub const RESET_CONFIG_CHANGED: u8 = 38;
    pub const READ_ADDITIONAL_STATUS: u8 = 48;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delimiter_values() {
        assert_eq!(delimiter::REQUEST_SHORT, 0x02);
        assert_eq!(delimiter::REQUEST_LONG, 0x82);
        assert_eq!(delimiter::RESPONSE_SHORT, 0x06);
        assert_eq!(delimiter::RESPONSE_LONG, 0x86);
        assert_eq!(delimiter::BURST_SHORT, 0x01);
        assert_eq!(delimiter::BURST_LONG, 0x81);
        assert_eq!(delimiter::LONG_ADDRESS_BIT, 0x80);
    }

    #[test]
    fn test_command_numbers() {
        assert_eq!(commands::READ_DEVICE_ID, 0);
        assert_eq!(commands::READ_PRIMARY_VARIABLE, 1);
        assert_eq!(commands::READ_LOOP_CURRENT_PERCENT, 2);
        assert_eq!(commands::READ_DYNAMIC_VARS, 3);
        assert_eq!(commands::WRITE_POLLING_ADDRESS, 6);
        assert_eq!(commands::READ_LOOP_CONFIG, 7);
        assert_eq!(commands::READ_DYNAMIC_VAR_CLASS, 8);
        assert_eq!(commands::READ_DEVICE_VARS_WITH_STATUS, 9);
        assert_eq!(commands::READ_UNIQUE_ID_BY_TAG, 11);
        assert_eq!(commands::READ_MESSAGE, 12);
        assert_eq!(commands::READ_TAG_DESCRIPTOR_DATE, 13);
        assert_eq!(commands::READ_PV_TRANSDUCER_INFO, 14);
        assert_eq!(commands::READ_DEVICE_INFO, 15);
        assert_eq!(commands::READ_FINAL_ASSEMBLY_NUMBER, 16);
        assert_eq!(commands::WRITE_MESSAGE, 17);
        assert_eq!(commands::WRITE_TAG_DESCRIPTOR_DATE, 18);
        assert_eq!(commands::WRITE_FINAL_ASSEMBLY_NUMBER, 19);
        assert_eq!(commands::READ_LONG_TAG, 20);
        assert_eq!(commands::READ_UNIQUE_ID_BY_LONG_TAG, 21);
        assert_eq!(commands::WRITE_LONG_TAG, 22);
        assert_eq!(commands::RESET_CONFIG_CHANGED, 38);
        assert_eq!(commands::READ_ADDITIONAL_STATUS, 48);
    }

    #[test]
    fn test_frame_constants() {
        assert_eq!(PREAMBLE_BYTE, 0xFF);
        assert_eq!(MIN_PREAMBLE_COUNT, 5);
        assert_eq!(MAX_PREAMBLE_COUNT, 20);
        assert_eq!(DEFAULT_PREAMBLE_COUNT, 10);
        assert_eq!(MAX_DATA_LENGTH, 255);
        assert_eq!(MAX_FRAME_LENGTH, 284);
    }

    #[test]
    fn test_uart_constants() {
        assert_eq!(BAUD_RATE, 1200);
        assert_eq!(RESPONSE_TIMEOUT_MS, 500);
        assert_eq!(RTS_HOLD_TIME_MS, 5);
    }
}
