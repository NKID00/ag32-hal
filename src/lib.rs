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

pub fn init(config: Config) -> Peripherals {
    critical_section::with(|cs| {});
    todo!()
}

pub use _gen::{Peripherals, peripherals};

pub(crate) mod _gen {
    use embassy_hal_internal::peripherals;

    include!(concat!(env!("OUT_DIR"), "/_gen.rs"));
}
