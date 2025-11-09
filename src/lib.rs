#![no_std]

mod dma;
mod fcb;
mod gpio;
mod rcc;

pub use ag32_pac as pac;

#[derive(Default)]
pub struct Config {
    pub rcc: rcc::Config,
}

pub fn init(config: Config) {
    critical_section::with(|cs| {})
}
