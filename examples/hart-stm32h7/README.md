# hart-stm32h7-example

Demonstrates polling a VEGAPULS 21 HART radar level transmitter using
[Embassy](https://embassy.dev/) on the STM32H745 Cortex-M4 core via the
AD5700-1 HART modem.

## Hardware required

- An STM32H7 development board (configured for the Cortex-M4 core)
- AD5700-1 HART modem (or compatible breakout)
- A HART-capable sensor, e.g. VEGAPULS 21

## Wiring

| Signal | Default pin | Notes                              |
|--------|-------------|------------------------------------|
| UART TX | PA0        | Connect to AD5700-1 TXD            |
| UART RX | PA1        | Connect to AD5700-1 RXD            |
| RTS    | PC0         | AD5700-1 /RTS (active-low TX gate) |
| CD     | PC1         | AD5700-1 CD (carrier detect)       |

## Customising

Open `src/main.rs` and follow the `// TODO:` comments to:

1. Change the UART peripheral instance (`UART4`).
2. Update TX/RX pin, DMA channel assignments.
3. Update the RTS output pin and CD input pin.
4. Change `poll_address` if your sensor is not at address 0.

## Flashing

Install [probe-rs](https://probe.rs/docs/getting-started/installation/) then:

```sh
cargo run --release
```

RTT logs appear in the probe-rs output.
