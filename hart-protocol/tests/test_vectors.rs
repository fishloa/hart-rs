/// Independent HART test vectors generated from yaq-project/hart-protocol (Python).
///
/// These were cross-validated against the yaq library's `tools.pack_command()`
/// and `tools.calculate_checksum()` functions. All request frames confirmed matching.
///
/// Response frames were manually constructed following the HART spec and
/// verified for checksum correctness.
///
/// Common long address used: manufacturer_id=0x1A, device_type=0x2B, device_id=0x112233
/// Address byte 0: 0x9A = primary_master(0x80) | manufacturer_id(0x1A)

// =========================================================================
// REQUEST FRAMES (cross-validated with yaq-project/hart-protocol Python lib)
// =========================================================================

/// Command 0 request, short addr 0, primary master
const REQ_CMD0_SHORT_PRIMARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // preamble (5)
    0x02, // delimiter: request short
    0x80, // address: primary master, poll addr 0
    0x00, // command 0
    0x00, // byte count 0
    0x82, // checksum
];

/// Command 0 request, short addr 0, secondary master
const REQ_CMD0_SHORT_SECONDARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02, // delimiter: request short
    0x00, // address: secondary master, poll addr 0
    0x00, // command 0
    0x00, // byte count 0
    0x02, // checksum
];

/// Command 0 request, long addr, primary master
const REQ_CMD0_LONG_PRIMARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x82, // delimiter: request long
    0x9A, 0x2B, 0x11, 0x22, 0x33, // long address
    0x00, // command 0
    0x00, // byte count 0
    0x33, // checksum
];

/// Command 1 request, long addr, primary master
const REQ_CMD1_LONG_PRIMARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x82, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x01, // command 1
    0x00, 0x32, // checksum
];

/// Command 3 request, long addr, primary master
const REQ_CMD3_LONG_PRIMARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x82, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x03, // command 3
    0x00, 0x30, // checksum
];

/// Command 48 request, long addr, primary master
const REQ_CMD48_LONG_PRIMARY: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x82, 0x9A, 0x2B, 0x11, 0x22, 0x33,
    0x30, // command 48 = 0x30
    0x00, 0x03, // checksum
];

// =========================================================================
// RESPONSE FRAMES (manually constructed, checksum verified against yaq lib)
// =========================================================================

/// Command 0 response: expansion=0xFE, mfr_id=0x1A, dev_type=0x2B,
/// preambles=5, hart_rev=7, dev_rev=1, sw_rev=3, hw_rev=4, flags=0,
/// device_id=0x112233
const RESP_CMD0_LONG: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x86, // delimiter: response long
    0x9A, 0x2B, 0x11, 0x22, 0x33, // long address
    0x00, // command 0
    0x0E, // byte count 14 (2 status + 12 data)
    0x00, 0x00, // status: no errors
    // --- Command 0 response data (12 bytes) ---
    0xFE, // expansion code
    0x1A, // manufacturer_id (in response this is the raw expanded_device_type high byte)
    0x2B, // device_type (expanded_device_type low byte)
    0x05, // min preambles
    0x07, // HART revision 7
    0x01, // device revision
    0x03, // software revision
    0x04, // hw_revision_and_signaling (hw=0, signaling=4... or hw=4 raw byte)
    0x00, // flags
    0x11, 0x22, 0x33, // device_id
    0xF2, // checksum
];

/// Command 1 response: PV = 3.14 meters
const RESP_CMD1_LONG: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x86, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x01, // command 1
    0x07, // byte count 7 (2 status + 5 data)
    0x00, 0x00, // status: no errors
    0x2D, // unit: meters (45 = 0x2D)
    0x40, 0x48, 0xF5, 0xC3, // 3.14 IEEE 754 big-endian
    0x22, // checksum
];

/// Command 2 response: current=12.5 mA, percent=53.125%
const RESP_CMD2_LONG: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x86, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x02, // command 2
    0x0A, // byte count 10 (2 status + 8 data)
    0x00, 0x00, // status
    0x41, 0x48, 0x00, 0x00, // 12.5 mA
    0x42, 0x54, 0x80, 0x00, // 53.125%
    0xA0, // checksum
];

/// Command 3 response: current=12.5mA, PV=53.125%(57), SV=2.5m(45),
/// TV=NaN/not_used(250), QV=25.3°C(32)
const RESP_CMD3_LONG: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x86, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x03, // command 3
    0x1A, // byte count 26 (2 status + 24 data)
    0x00, 0x00, // status
    // loop current: 12.5 mA
    0x41, 0x48, 0x00, 0x00, // PV: percent(57=0x39), 53.125
    0x39, 0x42, 0x54, 0x80, 0x00, // SV: meters(45=0x2D), 2.5
    0x2D, 0x40, 0x20, 0x00, 0x00, // TV: not_used(250=0xFA), NaN
    0xFA, 0x7F, 0xC0, 0x00, 0x00, // QV: celsius(32=0x20), 25.3
    0x20, 0x41, 0xCA, 0x66, 0x66, 0x2B, // checksum
];

/// Command 48 response: 5 device-specific status bytes
const RESP_CMD48_LONG: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x86, 0x9A, 0x2B, 0x11, 0x22, 0x33, 0x30, // command 48
    0x07, // byte count 7 (2 status + 5 data)
    0x00, 0x00, // status
    0x00, 0x01, 0x02, 0x03, 0x04, // device-specific status
    0x04, // checksum
];

// =========================================================================
// IEEE 754 big-endian float reference values
// =========================================================================
//       3.14 -> [0x40, 0x48, 0xF5, 0xC3]
//       12.5 -> [0x41, 0x48, 0x00, 0x00]
//     53.125 -> [0x42, 0x54, 0x80, 0x00]
//        2.5 -> [0x40, 0x20, 0x00, 0x00]
//       25.3 -> [0x41, 0xCA, 0x66, 0x66]
//        NaN -> [0x7F, 0xC0, 0x00, 0x00]

// =========================================================================
// TESTS: These verify that our Rust encoder/decoder produce byte-identical
// output to the independently generated vectors above.
// =========================================================================

use hart_protocol::commands::read_additional_status::{Cmd48Request, Cmd48Response};
use hart_protocol::commands::read_device_id::Cmd0Response;
use hart_protocol::commands::read_dynamic_vars::{Cmd3Request, Cmd3Response};
use hart_protocol::commands::read_loop_current::Cmd2Response;
use hart_protocol::commands::read_primary_variable::{Cmd1Request, Cmd1Response};
use hart_protocol::commands::{CommandRequest, CommandResponse};
use hart_protocol::consts::MIN_PREAMBLE_COUNT;
use hart_protocol::decode::Decoder;
use hart_protocol::encode::encode_frame;
use hart_protocol::types::{Address, FrameType, MasterRole};
use hart_protocol::units::UnitCode;

fn feed_all(decoder: &mut Decoder, bytes: &[u8]) -> Option<hart_protocol::decode::RawFrame> {
    for &b in bytes {
        match decoder.feed(b) {
            Ok(Some(frame)) => return Some(frame),
            Ok(None) => {}
            Err(e) => panic!("decode error: {:?}", e),
        }
    }
    None
}

// ---- Encoder test vectors ----

#[test]
fn test_vector_req_cmd0_short_primary() {
    let addr = Address::Short {
        master: MasterRole::Primary,
        burst: false,
        poll_address: 0,
    };
    let mut buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        0,
        &[],
        MIN_PREAMBLE_COUNT,
        &mut buf,
    )
    .unwrap();
    assert_eq!(&buf[..len], REQ_CMD0_SHORT_PRIMARY);
}

#[test]
fn test_vector_req_cmd0_short_secondary() {
    let addr = Address::Short {
        master: MasterRole::Secondary,
        burst: false,
        poll_address: 0,
    };
    let mut buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        0,
        &[],
        MIN_PREAMBLE_COUNT,
        &mut buf,
    )
    .unwrap();
    assert_eq!(&buf[..len], REQ_CMD0_SHORT_SECONDARY);
}

#[test]
fn test_vector_req_cmd0_long_primary() {
    let addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };
    let mut buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        0,
        &[],
        MIN_PREAMBLE_COUNT,
        &mut buf,
    )
    .unwrap();
    assert_eq!(&buf[..len], REQ_CMD0_LONG_PRIMARY);
}

#[test]
fn test_vector_req_cmd1_long_primary() {
    let addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };
    let req = Cmd1Request;
    let mut data_buf = [0u8; 4];
    let data_len = req.encode_data(&mut data_buf).unwrap();
    let mut frame_buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        Cmd1Request::COMMAND_NUMBER,
        &data_buf[..data_len],
        MIN_PREAMBLE_COUNT,
        &mut frame_buf,
    )
    .unwrap();
    assert_eq!(&frame_buf[..len], REQ_CMD1_LONG_PRIMARY);
}

#[test]
fn test_vector_req_cmd3_long_primary() {
    let addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };
    let req = Cmd3Request;
    let mut data_buf = [0u8; 4];
    let data_len = req.encode_data(&mut data_buf).unwrap();
    let mut frame_buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        Cmd3Request::COMMAND_NUMBER,
        &data_buf[..data_len],
        MIN_PREAMBLE_COUNT,
        &mut frame_buf,
    )
    .unwrap();
    assert_eq!(&frame_buf[..len], REQ_CMD3_LONG_PRIMARY);
}

#[test]
fn test_vector_req_cmd48_long_primary() {
    let addr = Address::Long {
        master: MasterRole::Primary,
        burst: false,
        manufacturer_id: 0x1A,
        device_type: 0x2B,
        device_id: 0x112233,
    };
    let req = Cmd48Request;
    let mut data_buf = [0u8; 4];
    let data_len = req.encode_data(&mut data_buf).unwrap();
    let mut frame_buf = [0u8; 32];
    let len = encode_frame(
        FrameType::Request,
        &addr,
        Cmd48Request::COMMAND_NUMBER,
        &data_buf[..data_len],
        MIN_PREAMBLE_COUNT,
        &mut frame_buf,
    )
    .unwrap();
    assert_eq!(&frame_buf[..len], REQ_CMD48_LONG_PRIMARY);
}

// ---- Decoder test vectors ----

#[test]
fn test_vector_decode_resp_cmd0_long() {
    let mut dec = Decoder::new();
    let frame = feed_all(&mut dec, RESP_CMD0_LONG).unwrap();
    assert_eq!(frame.command, 0);
    assert_eq!(frame.frame_type, hart_protocol::types::FrameType::Response);
    // byte_count = 0x0E = 14, so data.len() = 14 (2 status + 12 payload)
    assert_eq!(frame.data.len(), 14);
    // Strip 2 status bytes and decode
    let resp = Cmd0Response::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.expansion_code, 0xFE);
    assert_eq!(resp.expanded_device_type, 0x1A2B);
    assert_eq!(resp.min_preamble_count, 5);
    assert_eq!(resp.hart_revision, 7);
    assert_eq!(resp.device_revision, 1);
    assert_eq!(resp.software_revision, 3);
    assert_eq!(resp.hardware_revision, 0); // 0x04 >> 3 = 0
    assert_eq!(resp.physical_signaling, 4); // 0x04 & 0x07 = 4
    assert_eq!(resp.flags, 0);
    assert_eq!(resp.device_id, 0x112233);
}

#[test]
fn test_vector_decode_resp_cmd1_long() {
    let mut dec = Decoder::new();
    let frame = feed_all(&mut dec, RESP_CMD1_LONG).unwrap();
    assert_eq!(frame.command, 1);
    // byte_count = 0x07 = 7, data.len() = 7
    assert_eq!(frame.data.len(), 7);
    let resp = Cmd1Response::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.unit, UnitCode::Meters);
    let expected = f32::from_be_bytes([0x40, 0x48, 0xF5, 0xC3]);
    assert_eq!(resp.value, expected);
}

#[test]
fn test_vector_decode_resp_cmd2_long() {
    let mut dec = Decoder::new();
    let frame = feed_all(&mut dec, RESP_CMD2_LONG).unwrap();
    assert_eq!(frame.command, 2);
    // byte_count = 0x0A = 10
    assert_eq!(frame.data.len(), 10);
    let resp = Cmd2Response::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.current_ma, 12.5f32);
    assert_eq!(resp.percent_of_range, 53.125f32);
}

#[test]
fn test_vector_decode_resp_cmd3_long() {
    let mut dec = Decoder::new();
    let frame = feed_all(&mut dec, RESP_CMD3_LONG).unwrap();
    assert_eq!(frame.command, 3);
    // byte_count = 0x1A = 26
    assert_eq!(frame.data.len(), 26);
    let resp = Cmd3Response::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.loop_current_ma, 12.5f32);
    assert_eq!(resp.pv_unit, UnitCode::Percent);
    assert_eq!(resp.pv, 53.125f32);
    assert_eq!(resp.sv_unit, UnitCode::Meters);
    assert_eq!(resp.sv, 2.5f32);
    assert_eq!(resp.tv_unit, UnitCode::NotUsed);
    assert!(resp.tv.is_nan());
    assert_eq!(resp.qv_unit, UnitCode::DegreesCelsius);
    let expected_qv = f32::from_be_bytes([0x41, 0xCA, 0x66, 0x66]);
    assert_eq!(resp.qv, expected_qv);
}

#[test]
fn test_vector_decode_resp_cmd48_long() {
    let mut dec = Decoder::new();
    let frame = feed_all(&mut dec, RESP_CMD48_LONG).unwrap();
    assert_eq!(frame.command, 0x30); // 48 = 0x30
                                     // byte_count = 0x07 = 7
    assert_eq!(frame.data.len(), 7);
    let resp = Cmd48Response::decode_data(&frame.data[2..]).unwrap();
    assert_eq!(resp.data.as_slice(), &[0x00, 0x01, 0x02, 0x03, 0x04]);
}
