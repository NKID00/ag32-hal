#![no_std]
#![no_main]

use embassy_executor::Spawner;
use panic_halt as _;
use rtt_target::rtt_init_defmt;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    rtt_init_defmt!()
}

