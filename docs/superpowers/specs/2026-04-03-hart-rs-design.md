# hart-rs Design Specification

## Overview

A Rust implementation of the HART (Highway Addressable Remote Transducer) protocol for embedded systems. Targets STM32H7 (Cortex-M4) with an AD5700-1 HART modem, using the Embassy async runtime. Supports both master and slave roles, with master-first development driven by communication with a VEGAPULS 21 radar level sensor.

## Architecture

Three crates in a Cargo workspace monorepo (`hart-rs`):

```
hart-rs/
├── Cargo.toml                    (workspace)
├── hart-protocol/                (crates.io: hart-protocol)
├── ad5700/                       (crates.io: ad5700)
├── embassy-hart/                 (crates.io: embassy-hart)
└── examples/hart-stm32h7/        (not published)
```

### Dependency Graph

```
hart-protocol          (core only + heapless)
    ↑
ad5700                 (embedded-hal, embedded-hal-async, hart-protocol)
    │                   └── exports: Ad5700, Ad5700Async, HartMasterBlocking
    ↑
embassy-hart           (embassy-time, ad5700, hart-protocol)
                        └── exports: HartMaster (async)
```

### Crate Responsibilities

**`hart-protocol`** — Pure `no_std`, `no_alloc` codec library. Zero dependencies beyond `core` and `heapless`. Defines frame encoding/decoding, command types, data types (packed strings, unit codes, IEEE 754 floats big-endian). Usable on any platform including PC test harnesses. Contains no I/O.

**`ad5700`** — AD5700-1 HART modem driver. Generic over `embedded-hal` / `embedded-hal-async` traits. Handles RTS pin toggling for transmit/receive switching, CD pin monitoring, and UART framing at 1200 baud / 8-O-1. Provides both blocking (`Ad5700`) and async (`Ad5700Async`) modem APIs. Also exports `HartMasterBlocking` which combines the blocking modem with the `hart-protocol` codec for a synchronous master API.

**`embassy-hart`** — Async HART master (and later slave) built on `embassy-time` for timeouts. Combines `ad5700` async modem + `hart-protocol` codec into a typed async API. Embassy-specific due to `embassy-time` dependency.

**`examples/hart-stm32h7`** — Example firmware for STM32H7 M4 core + AD5700-1 + VEGAPULS 21. Uses Embassy runtime. Not published to crates.io.

## Design Principles

### No Magic Numbers

All protocol constants must be defined as named constants, enums, or derived from typed definitions. No raw integer literals in application code. This applies to:

- **Command numbers**: defined as associated constants on command types (`const COMMAND_NUMBER: u8 = ...`) and as a constants module (e.g., `commands::READ_DEVICE_ID`, `commands::READ_PRIMARY_VARIABLE`)
- **Delimiter bytes**: `delimiter::REQUEST_SHORT`, `delimiter::REQUEST_LONG`, `delimiter::RESPONSE_SHORT`, etc.
- **Unit codes**: `UnitCode::Meters`, `UnitCode::DegreesCelsius`, etc. — never raw `45u8` or `32u8`
- **Status bits**: named bitflags (e.g., `DeviceStatus::MALFUNCTION`, `DeviceStatus::CONFIG_CHANGED`)
- **Frame sizes**: `MAX_DATA_LENGTH`, `MAX_FRAME_LENGTH`, `PREAMBLE_BYTE`, `MIN_PREAMBLE_COUNT`, `DEFAULT_PREAMBLE_COUNT`
- **UART config**: `BAUD_RATE`, `RESPONSE_TIMEOUT_MS`, `RTS_HOLD_TIME_MS`
- **Address masks/shifts**: named constants for bit manipulation in address encoding

All constants live in a `consts` or appropriately named module within `hart-protocol`. Higher-level crates import and re-export as needed. Tests use the same constants — never duplicate literal values.

## HART Protocol Details

### Physical Layer

- Bell 202 FSK: 1200 Hz = mark (1), 2200 Hz = space (0)
- 1200 baud, 8 data bits, odd parity, 1 stop bit
- The AD5700-1 modem handles FSK modulation/demodulation; the MCU sees a standard UART interface
- MCU controls RTS (transmit enable) and reads CD (carrier detect)

### Frame Format

```
[Preamble (5-20 x 0xFF)] [Delimiter (1)] [Address (1 or 5)] [Command (1)] [Byte Count (1)] [Data (0-255)] [Checksum (1)]
```

- **Preamble**: 5-20 bytes of 0xFF for synchronization
- **Delimiter**: encodes frame type and addressing mode, defined as constants:
  - `delimiter::REQUEST_SHORT` (0x02)
  - `delimiter::REQUEST_LONG` (0x82)
  - `delimiter::RESPONSE_SHORT` (0x06)
  - `delimiter::RESPONSE_LONG` (0x86)
  - `delimiter::BURST_SHORT` / `delimiter::BURST_LONG` (0x01 / 0x81)
- **Address**: 1 byte (short) or 5 bytes (long). Contains master role bit, burst bit, and device address.
- **Command**: 1 byte command number
- **Byte Count**: length of data field
- **Data**: command-specific payload (0-255 bytes)
- **Checksum**: XOR of all bytes from delimiter through last data byte

### Addressing

**Short address (1 byte):**
- Bit 7: primary master flag
- Bit 6: burst mode indicator
- Bits 3-0: poll address (0-15)

**Long address (5 bytes):**
- Byte 0: manufacturer ID with master/burst flags in high bits
- Byte 1: manufacturer device type
- Bytes 2-4: device serial number (24-bit, big endian)

### Response Status

Every response includes two status bytes:

- **Byte 0 (Communication):** bit 7 = error flag, remaining bits indicate specific communication errors
- **Byte 1 (Device Status):** malfunction, configuration changed, cold start, PV out of limits, etc.

### Data Type Encoding

- **Float**: IEEE 754, big endian (4 bytes)
- **Packed strings**: 6-bit encoding, 4 characters per 3 bytes (used for tags, messages)
- **Unit codes**: single byte preceding each variable value

## Commands

### Phase 1 — Talk to the VEGAPULS 21

| Cmd | Name | Direction | Purpose |
|-----|------|-----------|---------|
| 0 | Read Device ID | Read | Discover device, get long address, learn preamble count |
| 1 | Read Primary Variable | Read | Read level (lin. percent on VEGAPULS 21) |
| 2 | Read Loop Current & % of Range | Read | Read 4-20mA current and percentage |
| 3 | Read Dynamic Variables & Loop Current | Read | Read all 4 variables: PV (lin. percent), SV (distance), TV (meas. reliability), QV (electronics temp) |
| 48 | Read Additional Device Status | Read | Read device diagnostics (fault codes, status bits) |

### Phase 2 — Full Read-Only Access

| Cmd | Name | Direction | Purpose |
|-----|------|-----------|---------|
| 7 | Read Loop Configuration | Read | Polling address and loop current mode |
| 8 | Read Dynamic Variable Classifications | Read | PV/SV/TV/QV type classifications |
| 9 | Read Device Variables with Status | Read | Extended variable access with per-variable status |
| 11 | Read Unique ID by Tag | Read | Find device by its tag string |
| 12 | Read Message | Read | 32-char device message |
| 13 | Read Tag, Descriptor, Date | Read | Device tag (8 char), descriptor (16 char), date |
| 14 | Read PV Transducer Info | Read | Sensor limits, minimum span, unit code |
| 15 | Read Device Information | Read | Output range, damping, write protect, PV alarm codes |
| 16 | Read Final Assembly Number | Read | 3-byte assembly number |
| 20 | Read Long Tag | Read | 32-char long tag (HART 6+) |

### Phase 3 — Configuration

| Cmd | Name | Direction | Purpose |
|-----|------|-----------|---------|
| 6 | Write Polling Address | Write | Set multidrop address |
| 17 | Write Message | Write | Set 32-char device message |
| 18 | Write Tag, Descriptor, Date | Write | Set tag, descriptor, date |
| 19 | Write Final Assembly Number | Write | Set assembly number |
| 22 | Write Long Tag | Write | Set 32-char long tag |
| 38 | Reset Configuration Changed Flag | Write | Clear config-changed status bit |

### Universal Command Gaps

Commands 4, 5, and 10 are reserved/unassigned in the HART specification and will not be implemented.

## Core Types

### Frame Layer (`hart-protocol`)

```rust
enum Address {
    Short { master: MasterRole, burst: bool, poll_address: u8 },
    Long { master: MasterRole, burst: bool, manufacturer_id: u8, device_type: u8, device_id: u32 },
}

enum MasterRole { Primary, Secondary }
enum FrameType { Request, Response, Burst }

struct Frame<'a> {
    frame_type: FrameType,
    address: Address,
    command: u8,
    data: &'a [u8],
}

// Stateless encoder — writes frame into caller's buffer
struct Encoder;
impl Encoder {
    fn encode(frame: &Frame, preamble_count: u8, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

// Stateful byte-at-a-time decoder — works with interrupt-driven UART
struct Decoder { state: DecoderState }
impl Decoder {
    fn feed(&mut self, byte: u8) -> Result<Option<RawFrame>, DecodeError>;
    fn reset(&mut self);
}

// Decoded frame with owned data
struct RawFrame {
    frame_type: FrameType,
    address: Address,
    command: u8,
    status: [u8; 2],
    data: heapless::Vec<u8, 256>,
}

struct ResponseStatus {
    communication_error: CommunicationError,
    device_status: DeviceStatus,
}
```

### Command Layer (`hart-protocol`)

```rust
trait CommandRequest {
    const COMMAND_NUMBER: u8;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

trait CommandResponse: Sized {
    const COMMAND_NUMBER: u8;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError>;
}

// Command numbers also available as a constants module for use
// in match arms, logging, etc:
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

/// Unit codes — each variant carries its wire value as a constant,
/// accessible via `UnitCode::as_u8()` / `UnitCode::from_u8()`.
/// No raw integers in calling code.
enum UnitCode {
    Meters,            // wire: 45
    Millimeters,       // wire: 35
    Percent,           // wire: 57
    DegreesCelsius,    // wire: 32
    DegreesFahrenheit, // wire: 33
    Bar,               // wire: 7
    Millibar,          // wire: 8
    KiloPascals,       // wire: 12
    InchesWaterColumn, // wire: 1
    Psi,               // wire: 6
    // ... extensible
    Unknown(u8),       // fallback for unrecognized codes
}
```

Each command in phases 1-3 gets a typed request struct (often empty for read commands) and a typed response struct implementing these traits.

### AD5700 Driver (`ad5700`)

```rust
// Blocking
pub struct Ad5700<UART, RTS, CD> { uart, rts, cd }
impl Ad5700 {
    fn new(uart, rts, cd) -> Self;
    fn transmit(&mut self, data: &[u8]) -> Result<(), Ad5700Error>;
    fn receive(&mut self, buf: &mut [u8], timeout_ms: u32) -> Result<usize, Ad5700Error>;
    fn carrier_detected(&self) -> Result<bool, Ad5700Error>;
    fn release(self) -> (UART, RTS, CD);
}

// Async
pub struct Ad5700Async<UART, RTS, CD> { uart, rts, cd }
impl Ad5700Async {
    async fn transmit(&mut self, data: &[u8]) -> Result<(), Ad5700Error>;
    async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, Ad5700Error>;
}

pub enum Ad5700Error { Uart, NoCarrier, Timeout }
```

### Blocking Master (`ad5700`)

```rust
pub struct HartMasterBlocking<UART, RTS, CD> {
    modem: Ad5700<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; 280],
    rx_buf: [u8; 280],
    preamble_count: u8,
}
impl HartMasterBlocking {
    fn send_command<Req, Resp>(&mut self, address: &Address, request: &Req)
        -> Result<(ResponseStatus, Resp), HartError>;
    fn read_device_id(&mut self, poll_address: u8) -> Result<(Address, ReadDeviceIdResponse), HartError>;
    fn read_pv(&mut self, address: &Address) -> Result<ReadPvResponse, HartError>;
    fn read_dynamic_vars(&mut self, address: &Address) -> Result<ReadDynamicVarsResponse, HartError>;
    fn read_additional_status(&mut self, address: &Address) -> Result<ReadAdditionalStatusResponse, HartError>;
}
```

### Async Master (`embassy-hart`)

```rust
pub struct HartMaster<UART, RTS, CD> {
    modem: Ad5700Async<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; 280],
    rx_buf: [u8; 280],
    preamble_count: u8,
}
impl HartMaster {
    async fn send_command<Req, Resp>(&mut self, address: &Address, request: &Req)
        -> Result<(ResponseStatus, Resp), HartError>;
    async fn read_device_id(&mut self, poll_address: u8) -> Result<(Address, ReadDeviceIdResponse), HartError>;
    async fn read_pv(&mut self, address: &Address) -> Result<ReadPvResponse, HartError>;
    async fn read_dynamic_vars(&mut self, address: &Address) -> Result<ReadDynamicVarsResponse, HartError>;
    async fn read_additional_status(&mut self, address: &Address) -> Result<ReadAdditionalStatusResponse, HartError>;
}

pub enum HartError { Modem(Ad5700Error), Encode(EncodeError), Decode(DecodeError), CommandError(ResponseStatus), Timeout }
```

## VEGAPULS 21 Specifics

The VEGAPULS 21 is an 80 GHz FMCW radar level sensor (4-20mA/HART, two-wire).

### Dynamic Variables

| Variable | Default Assignment | Typical Unit |
|----------|-------------------|--------------|
| PV | Linearized percent | % (57) |
| SV | Distance | m (45) |
| TV | Measurement reliability | — |
| QV | Electronics temperature | deg C (32) |

### Device Status (Command 48)

The device reports extensive diagnostics via Command 48 response bytes. Status codes include:
- F013: No measured value available
- F017: Adjustment span too small
- F025: Linearization table error
- F036: No operable software
- F040: Hardware defect
- F080: General software error
- F105: Determining measured value (switch-on phase)
- F113: EMC interference
- F125: Impermissible electronics temperature
- F260-F265: Calibration, setup, installation, measurement errors
- S600-S603: Out-of-spec temperature, overfilling, low voltage
- M500-M507: Maintenance conditions
- C700: Simulation active

## Testing Strategy

| Layer | Method | Environment |
|-------|--------|-------------|
| `hart-protocol` codec | Unit tests: encode/decode roundtrips, known byte sequences from reference implementations, edge cases (max data, short/long address, checksum errors, malformed preambles) | `cargo test` on host |
| `hart-protocol` decoder | State machine tests: byte-at-a-time feeding, garbage input, partial frames, truncated frames, double preambles | `cargo test` on host |
| `ad5700` driver | `embedded-hal-mock` for UART/GPIO mocking. Verify RTS timing, transmit/receive sequencing, carrier detect | `cargo test` on host |
| `embassy-hart` master | Mock `Ad5700Async` with canned responses. Verify discovery flow, command roundtrips, timeout handling, error propagation | `cargo test` on host |
| Integration | Real STM32H7 + AD5700-1 + VEGAPULS 21. Verify Command 0 discovery, Command 3 level readings, Command 48 diagnostics | On hardware |

## Reference Implementations

Studied during design (not dependencies):

- **[Hart-Master-Slave-8.0](https://github.com/BorstAutomation/Hart-Master-Slave-8.0)** (C++, Apache 2.0) — Full master+slave, layered architecture. Primary architectural reference.
- **[yaq-project/hart-protocol](https://github.com/yaq-project/hart-protocol)** (Python) — Sans-I/O HART codec. Reference for command field layouts and byte ordering.
- **[profirust](https://github.com/Rahix/profirust)** (Rust) — PROFIBUS stack. Reference for embedded-hal PHY trait patterns.
- **[Analog Devices EVAL-ADICUP3029 ad5700.c](https://github.com/analogdevicesinc/EVAL-ADICUP3029)** — Authoritative AD5700 modem control reference.
- **[atat](https://crates.io/crates/atat)** / **[crsf](https://crates.io/crates/crsf)** (Rust) — Reference for byte-at-a-time `no_std` UART parsers.

## Non-Goals

- WirelessHART (IEEE 802.15.4) — different protocol entirely
- HART-IP (TCP/UDP transport) — may be added later but not in scope
- DD (Device Description) file parsing — device-specific commands are added manually
- Bell 202 FSK in software — the AD5700-1 handles modulation
