//! STM32H7 + AD5700-1 HART master example — VEGAPULS 21 level sensor.
//!
//! Demonstrates polling a VEGAPULS 21 radar level transmitter over HART
//! using Embassy on the STM32H745 Cortex-M4 core.
//!
//! # Customising this example
//!
//! 1. Change the `embassy_stm32::init` peripherals below to match your board.
//! 2. Update the UART instance (`UART4` below) and pin assignments as needed.
//! 3. Update the RTS and CD pin assignments to your wiring.
//! 4. If your sensor uses a different poll address, change `poll_address` in
//!    the `Address::Short` literal.
//! 5. Flash with `cargo run --release` (requires probe-rs on PATH).

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::usart::{Config as UartConfig, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::{Duration, Timer};
use panic_probe as _;

use ad5700::asynch::Ad5700Async;
use embassy_hart::master::HartMaster;
use hart_protocol::commands::read_device_id::{Cmd0Request, Cmd0Response};
use hart_protocol::commands::read_dynamic_vars::{Cmd3Request, Cmd3Response};
use hart_protocol::types::{Address, MasterRole};

// ---------------------------------------------------------------------------
// Interrupt binding — change UART4 to whichever USART/UART instance you use.
// ---------------------------------------------------------------------------
bind_interrupts!(struct Irqs {
    // TODO: Change UART4 to your UART instance (e.g. USART1, USART2, UART5 …)
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // -----------------------------------------------------------------------
    // Clock & peripheral init — embassy-stm32 reads the chip's RCC registers
    // and sets up the default clock tree.  Adjust `embassy_stm32::Config` if
    // you need a specific PLL configuration.
    // -----------------------------------------------------------------------
    let p = embassy_stm32::init(Default::default());

    // -----------------------------------------------------------------------
    // UART — 1200 baud, 8 data bits, odd parity, 1 stop bit (HART spec).
    //
    // TODO: Change UART4, PA0 (TX), PA1 (RX) to your pin assignment.
    // -----------------------------------------------------------------------
    let mut uart_cfg = UartConfig::default();
    uart_cfg.baudrate = 1200;
    uart_cfg.parity = embassy_stm32::usart::Parity::ParityOdd;
    uart_cfg.stop_bits = embassy_stm32::usart::StopBits::STOP1;
    uart_cfg.data_bits = embassy_stm32::usart::DataBits::DataBits8;

    let uart = Uart::new(
        p.UART4,        // TODO: Change to your UART instance
        p.PA1,          // TODO: Change to your RX pin
        p.PA0,          // TODO: Change to your TX pin
        Irqs,
        p.DMA1_CH0,     // TODO: Change to a free TX DMA channel
        p.DMA1_CH1,     // TODO: Change to a free RX DMA channel
        uart_cfg,
    )
    .expect("UART init failed");

    // -----------------------------------------------------------------------
    // RTS — drives the AD5700-1 /RTS pin (active-low transmit enable).
    //
    // TODO: Change PC0 to your RTS pin assignment.
    // -----------------------------------------------------------------------
    let rts = Output::new(
        p.PC0, // TODO: Change to your RTS pin
        Level::Low,
        Speed::Low,
    );

    // -----------------------------------------------------------------------
    // CD (Carrier Detect) — reads the AD5700-1 CD output (active-high when a
    // HART carrier is present on the loop).
    //
    // TODO: Change PC1 to your CD pin assignment.
    // -----------------------------------------------------------------------
    let cd = Input::new(
        p.PC1, // TODO: Change to your CD pin
        Pull::Down,
    );

    // -----------------------------------------------------------------------
    // Build the modem and HART master.
    // -----------------------------------------------------------------------
    let modem = Ad5700Async::new(uart, rts, cd);
    let mut master = HartMaster::new(modem);

    // Short address — primary master, poll address 0 (default for most HART
    // devices fresh from the factory).
    // TODO: Change poll_address if your sensor is configured differently.
    let addr = Address::Short {
        master: MasterRole::Primary,
        burst: false,
        poll_address: 0,
    };

    info!("HART master ready — polling VEGAPULS 21 at poll address 0");

    loop {
        // -------------------------------------------------------------------
        // Command 0 — Read Unique Identifier
        // Discovers the device and retrieves firmware/hardware revisions plus
        // the minimum preamble count the device requires.
        // -------------------------------------------------------------------
        match master.send_command::<Cmd0Request, Cmd0Response>(&addr, &Cmd0Request).await {
            Ok((status, resp)) => {
                if status.has_error() {
                    warn!("Cmd0 comm error: status={:#04x} {:#04x}", status.byte0, status.byte1);
                } else {
                    info!(
                        "Device found: type={:#06x} id={:#08x} hart_rev={} sw_rev={} preambles={}",
                        resp.expanded_device_type,
                        resp.device_id,
                        resp.hart_revision,
                        resp.software_revision,
                        resp.min_preamble_count,
                    );

                    // Honour the device's requested preamble count so
                    // subsequent commands are reliably decoded.
                    master.set_preamble_count(resp.min_preamble_count.max(5));
                }
            }
            Err(e) => {
                error!("Cmd0 failed: {:?}", defmt::Debug2Format(&e));
            }
        }

        // -------------------------------------------------------------------
        // Command 3 — Read Dynamic Variables and Loop Current
        // Returns PV (level %), SV (distance m), TV (unused), QV (temp °C).
        // -------------------------------------------------------------------
        match master.send_command::<Cmd3Request, Cmd3Response>(&addr, &Cmd3Request).await {
            Ok((status, resp)) => {
                if status.has_error() {
                    warn!("Cmd3 comm error: status={:#04x} {:#04x}", status.byte0, status.byte1);
                } else {
                    // VEGAPULS 21 variable mapping:
                    //   PV  = level           (%)
                    //   SV  = distance        (m)
                    //   TV  = not used        (NaN)
                    //   QV  = temperature     (°C)
                    info!(
                        "Loop current: {} mA",
                        resp.loop_current_ma,
                    );
                    info!(
                        "PV (level):    {} {:?}",
                        resp.pv,
                        defmt::Debug2Format(&resp.pv_unit),
                    );
                    info!(
                        "SV (distance): {} {:?}",
                        resp.sv,
                        defmt::Debug2Format(&resp.sv_unit),
                    );
                    info!(
                        "TV:            {} {:?}",
                        resp.tv,
                        defmt::Debug2Format(&resp.tv_unit),
                    );
                    info!(
                        "QV (temp):     {} {:?}",
                        resp.qv,
                        defmt::Debug2Format(&resp.qv_unit),
                    );

                    if status.device_malfunction() {
                        warn!("Device reports malfunction!");
                    }
                    if status.pv_out_of_limits() {
                        warn!("PV out of limits!");
                    }
                }
            }
            Err(e) => {
                error!("Cmd3 failed: {:?}", defmt::Debug2Format(&e));
            }
        }

        // Poll every 2 seconds.
        Timer::after(Duration::from_secs(2)).await;
    }
}
