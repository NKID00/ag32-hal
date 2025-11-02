/// Reset and Clock Control (RCC) configuration
///
/// PLL and AHB clock settings cannot be read or written directly as they are configured along side FPGA via the FCB0 peripheral.
pub struct Config {
    pub hse: Option<Hse>,
    pub sys: Sysclk,
    pub apb_pre: APBPrescaler,
}

impl Config {
    pub const SYSCLK_FREQ_100MHZ_HSI: Config = {};
}

impl Default for Config {
    fn default() -> Self {
        Self::SYSCLK_FREQ_100MHZ_HSI
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HseMode {
    Oscillator,
    Bypass,
}

#[derive(Debug, Clone, Copy)]
pub struct Hse {
    pub freq: Hertz,
    pub mode: HseMode,
}
