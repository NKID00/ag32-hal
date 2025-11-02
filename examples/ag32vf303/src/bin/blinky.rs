#![no_std]
#![no_main]

use ag32_hal as hal;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use hal::gpio::{AnyPin, Level, Output, Pin};
use hal::pac;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_defmt};

#[embassy_executor::main]
async fn main(_s: Spawner) {
    rtt_init_defmt!();
    let mut config = hal::Config::default();
    config.rcc = hal::rcc::Config::SYSCLK_FREQ_100MHZ_HSI;
    let p = hal::init(config);
    spawner.spawn(blink(p.PA10.degrade(), 500)).unwrap();
    loop {
        Timer::after_millis(1000).await;
        rprintln!("systick: {}", pac::SYSTICK.cnt().read());
    }
}

#[embassy_executor::task]
async fn blink(pin: AnyPin, interval_ms: u64) {
    let mut led = Output::new(pin, Level::Low, Default::default());

    loop {
        led.set_high();
        Timer::after_millis(interval_ms).await;
        led.set_low();
        Timer::after(interval_ms).await;
    }
}
