# hart-rs

A `no_std` HART protocol implementation for embedded systems, written in Rust.

[![CI](https://github.com/fishloa/hart-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fishloa/hart-rs/actions/workflows/ci.yml)

## What it is

HART (Highway Addressable Remote Transducer) is an industrial protocol layered
over a 4-20 mA current loop. This workspace implements the full protocol stack
— codec, modem driver, and async master — as three `no_std` crates targeting
embedded microcontrollers such as the STM32H7 with an AD5700-1 HART modem.

## Crate architecture

```
hart-protocol          (codec: encode, decode, typed commands)
      ^
      |
   ad5700              (AD5700-1 modem driver: blocking + async)
      ^
      |
embassy-hart           (async HART master via Embassy + embassy-time)
```

`hart-protocol` is a standalone codec with no hardware dependencies.
`ad5700` wraps the modem with `embedded-hal` traits and uses `hart-protocol`.
`embassy-hart` builds on `ad5700`'s async driver and adds timeout management.

## Supported commands

### Universal read commands

| Cmd | Description                          |
|-----|--------------------------------------|
| 0   | Read unique identifier (device ID)   |
| 1   | Read primary variable                |
| 2   | Read loop current and percent range  |
| 3   | Read dynamic variables               |

### Common-practice read commands

| Cmd | Description                          |
|-----|--------------------------------------|
| 9   | Read device variables with status    |
| 11  | Read unique identifier by tag        |
| 12  | Read message                         |
| 13  | Read tag, descriptor, and date       |
| 14  | Read primary variable transducer info|
| 15  | Read device info                     |
| 16  | Read final assembly number           |
| 20  | Read long tag                        |
| 48  | Read additional device status        |

### Write commands

| Cmd | Description                          |
|-----|--------------------------------------|
| 6   | Write polling address                |
| 17  | Write message                        |
| 18  | Write tag, descriptor, and date      |
| 19  | Write final assembly number          |
| 22  | Write long tag                       |
| 38  | Reset configuration changed flag     |

## Quick start

Encode a Command 0 request frame and decode a response using `hart-protocol`
(no hardware or I/O involved):

```rust
use hart_protocol::encode::encode_frame;
use hart_protocol::decode::Decoder;
use hart_protocol::types::{Address, FrameType, MasterRole};
use hart_protocol::commands::read_device_id::{Cmd0Request, Cmd0Response};
use hart_protocol::commands::{CommandRequest, CommandResponse};
use hart_protocol::consts::MIN_PREAMBLE_COUNT;

// --- Encode a Command 0 request ---
let address = Address::Short {
    master: MasterRole::Primary,
    burst: false,
    poll_address: 0,
};

let req = Cmd0Request;
let mut data_buf = [0u8; 0];
let data_len = req.encode_data(&mut data_buf).unwrap();

let mut frame_buf = [0u8; 32];
let frame_len = encode_frame(
    FrameType::Request,
    &address,
    Cmd0Request::COMMAND_NUMBER,
    &data_buf[..data_len],
    MIN_PREAMBLE_COUNT,
    &mut frame_buf,
).unwrap();

// frame_buf[..frame_len] is ready to send to the modem.

// --- Decode a Command 0 response byte-by-byte ---
let raw_response: &[u8] = &[
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF,       // preamble
    0x06,                               // delimiter: response short
    0x80,                               // address
    0x00,                               // command 0
    0x0E,                               // byte count (14 = 2 status + 12 data)
    0x00, 0x00,                         // response status bytes
    0xFE, 0x1A, 0x2B, 0x05, 0x07,
    0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33, // device identity
    0x00,                               // checksum (illustrative)
];

let mut decoder = Decoder::new();
let mut raw_frame = None;
for &byte in raw_response {
    if let Some(frame) = decoder.feed(byte).unwrap_or(None) {
        raw_frame = Some(frame);
        break;
    }
}

if let Some(frame) = raw_frame {
    // Strip the two response status bytes before decoding the command payload.
    let payload = &frame.data[2..];
    let resp = Cmd0Response::decode_data(payload).unwrap();
    let _ = resp.device_id;
}
```

## Hardware target

This stack is designed for the **STM32H7** microcontroller paired with the
**Analog Devices AD5700-1** HART modem. The `ad5700` and `embassy-hart` crates
depend on `embedded-hal` and `Embassy` respectively, keeping the codec
(`hart-protocol`) portable to any platform.

## Releasing

1. Update version in root `Cargo.toml` (`[workspace.package] version = "X.Y.Z"`)
2. Commit: `release: vX.Y.Z`
3. Tag: `git tag vX.Y.Z`
4. Push: `git push && git push --tags`

CI will verify tests pass, check the tag matches Cargo.toml, and publish all crates to crates.io in dependency order.

Requires `CARGO_REGISTRY_TOKEN` secret in GitHub repository settings.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
