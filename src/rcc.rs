use crate::pac::sys::clk_cntl::ClkSource as Sysclk;

/// Reset and Clock Control (RCC) configuration
pub struct Config {
    pub sys: Sysclk,
}

impl Config {
    pub fn new() -> Self {
        Config { sys: Sysclk::Hsi }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
