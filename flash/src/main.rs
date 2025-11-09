#![no_std]
#![no_main]

use core::slice::from_raw_parts_mut;

use ag32_pac as pac;
use flash_algorithm::*;
use riscv::register::mie::Mie;
use rtt_target::{rprintln, rtt_init_print};

struct Algorithm;

algorithm!(Algorithm, {
    device_name: "ag32vf303",
    device_type: DeviceType::Onchip,
    flash_address: 0x80000000,
    flash_size: 0x40000,
    page_size: 0x100,
    empty_value: 0xFF,
    program_time_out: 100,
    erase_time_out: 2000,
    sectors: [{
        size: 0x1000,
        address: 0x0,
    }]
});

const FLASH_KEY1: u32 = 0x45670123;
const FLASH_KEY2: u32 = 0xCDEF89AB;

#[inline]
fn flash_unlock(flash: &pac::Flash) {
    if flash.flash_cr().read().flash_cr_lock().bit_is_set() {
        flash.flash_keyr().write(|w| unsafe { w.bits(FLASH_KEY1) });
        flash.flash_keyr().write(|w| unsafe { w.bits(FLASH_KEY2) });
    }
    if flash.flash_cr().read().flash_cr_optwre().bit_is_clear() {
        flash
            .flash_optkeyr()
            .write(|w| unsafe { w.bits(FLASH_KEY1) });
        flash
            .flash_optkeyr()
            .write(|w| unsafe { w.bits(FLASH_KEY2) });
    }
}

#[inline]
fn flash_start(flash: &pac::Flash) {
    riscv::asm::delay(256);
    flash.flash_cr().modify(|_, w| w.flash_cr_strt().set_bit());
}

#[inline]
fn flash_wait_for_busy(flash: &pac::Flash) {
    while flash.flash_sr().read().flash_sr_bsy().bit_is_set() {
        core::hint::spin_loop();
    }
}

#[inline]
fn flash_flex_read(
    flash: &pac::Flash,
    cmd: u8,
    has_addr: bool,
    address: u32,
    data_bytes: u8,
    dummy_bytes: u8,
) -> u32 {
    let read_ctrl = ((has_addr as u32) << 31)
        | ((dummy_bytes as u32) << 16)
        | ((data_bytes as u32) << 8)
        | ((cmd as u32) << 0);
    flash
        .flash_read_ctrl()
        .write(|w| unsafe { w.bits(read_ctrl) });
    if has_addr {
        flash.flash_ar().write(|w| unsafe { w.bits(address) });
    }
    flash.flash_cr().modify(|_, w| w.flash_cr_read().set_bit());
    flash_start(flash);
    flash_wait_for_busy(flash);
    flash
        .flash_cr()
        .modify(|_, w| w.flash_cr_read().clear_bit());
    flash.flash_read_data().read().bits()
}

#[inline]
fn flash_capacity_bytes(flash: &pac::Flash) -> u32 {
    let bits = flash_flex_read(flash, 0x5a, true, 0x34, 4, 1);
    (bits + 1) >> 3
}

#[inline]
fn flash_unique_id(flash: &pac::Flash) -> [u32; 4] {
    [4, 8, 12, 16].map(|i| flash_flex_read(flash, 0x4b, false, 0, i, 4))
}

impl FlashAlgorithm for Algorithm {
    fn new(_address: u32, _clock: u32, _function: Function) -> Result<Self, ErrorCode> {
        rtt_init_print!();
        rprintln!("Initialize flash algorithm");
        let p = unsafe { pac::Peripherals::steal() };
        p.sys.clk_cntl().modify(|_, w| {
            unsafe { w.sclk_div().bits(0) };
            w.clk_source().hsi()
        });
        p.plic.disable();
        unsafe { riscv::register::mie::write(Mie::from_bits(0)) };
        flash_unlock(&p.flash);
        rprintln!("Flash capacity: {} bytes", flash_capacity_bytes(&p.flash));
        rprintln!("Flash unique id: {:08x?}", flash_unique_id(&p.flash));
        rprintln!("Initialized flash algorithm");
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        rprintln!("Erase All");
        let flash = unsafe { pac::Flash::steal() };
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_per().clear_bit();
            w.flash_cr_mer().set_bit();
            w.flash_cr_ber().clear_bit()
        });
        flash_start(&flash);
        flash_wait_for_busy(&flash);
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_per().clear_bit();
            w.flash_cr_mer().clear_bit();
            w.flash_cr_ber().clear_bit()
        });
        Ok(())
    }

    fn erase_sector(&mut self, addr: u32) -> Result<(), ErrorCode> {
        rprintln!("Erase sector addr: {:#08x}", addr);
        let flash = unsafe { pac::Flash::steal() };
        flash.flash_ar().write(|w| unsafe { w.bits(addr) });
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_per().clear_bit();
            w.flash_cr_mer().clear_bit();
            w.flash_cr_ber().set_bit()
        });
        flash_start(&flash);
        flash_wait_for_busy(&flash);
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_per().clear_bit();
            w.flash_cr_mer().clear_bit();
            w.flash_cr_ber().clear_bit()
        });
        Ok(())
    }

    fn program_page(&mut self, addr: u32, data: &[u8]) -> Result<(), ErrorCode> {
        rprintln!("Program page addr: {:#08x}, size: {}", addr, data.len());
        let flash = unsafe { pac::Flash::steal() };
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_pg().set_bit();
            w.flash_cr_fastpg().set_bit()
        });
        let dst = unsafe { from_raw_parts_mut(addr as *mut u8, data.len()) };
        dst.copy_from_slice(data);
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_pg().clear_bit();
            w.flash_cr_fastpg().clear_bit();
            w.flash_cr_optpg().clear_bit()
        });
        flash_wait_for_busy(&flash);
        Ok(())
    }
}
