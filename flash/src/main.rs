#![no_std]
#![no_main]

#[cfg(all(feature = "no-rtt", feature = "rtt"))]
compile_error!("features \"no-rtt\" and \"rtt\" cannot be enabled simultaneously");
#[cfg(not(any(feature = "no-rtt", feature = "rtt")))]
compile_error!("one of features \"no-rtt\" and \"rtt\" must be enabled");

use core::slice::{from_raw_parts, from_raw_parts_mut};

use ag32_pac as pac;
use flash_algorithm::*;
#[cfg(feature = "dma")]
use pac::Dmac0;
use pac::Flash;
#[cfg(feature = "no-rtt")]
use panic_halt as _;
use riscv::register::mie::Mie;
#[cfg(feature = "rtt")]
use rtt_target::{rprint, rprintln, rtt_init_print};

struct Algorithm;

algorithm!(Algorithm, {
    device_name: "ag32vf303",
    device_type: DeviceType::Onchip,
    flash_address: 0x80000000,
    flash_size: 0x40000,
    page_size: 0x1000,  // actual page size is 256 bytes, raised to 4K to amortize bottleneck from writing to memory
    empty_value: 0xFF,
    program_time_out: 10000,
    erase_time_out: 1000,
    sectors: [{
        size: 0x1000,
        address: 0x0,
    }]
});

const FLASH_KEY1: u32 = 0x45670123;
const FLASH_KEY2: u32 = 0xCDEF89AB;

#[inline]
fn flash_unlock(flash: &Flash) {
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
fn flash_start(flash: &Flash) {
    riscv::asm::delay(256);
    flash.flash_cr().modify(|_, w| w.flash_cr_strt().set_bit());
}

#[inline]
fn flash_wait_for_busy(flash: &Flash) {
    while flash.flash_sr().read().flash_sr_bsy().bit_is_set() {
        core::hint::spin_loop();
    }
}

#[cfg(feature = "rtt")]
#[inline]
fn flash_flex_read(
    flash: &Flash,
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

#[cfg(feature = "rtt")]
#[inline]
fn flash_capacity_bytes(flash: &Flash) -> u32 {
    let bits = flash_flex_read(flash, 0x5a, true, 0x34, 4, 1);
    (bits + 1) >> 3
}

#[cfg(feature = "rtt")]
#[inline]
fn flash_unique_id(flash: &Flash) -> [u32; 4] {
    [4, 8, 12, 16].map(|i| flash_flex_read(flash, 0x4b, false, 0, i, 4))
}

#[cfg(feature = "dma")]
#[inline]
fn flash_program_dma<const N: usize>(dmac: &Dmac0, src: &[u32; N], dst: &mut [u32; N]) {
    flash_program_dma_raw(dmac, src.as_ptr(), dst.as_mut_ptr(), N);
}

#[cfg(feature = "dma")]
#[inline]
fn flash_program_dma_raw(dmac: &Dmac0, src: *const u32, dst: *mut u32, transfer_size: usize) {
    #[cfg(feature = "rtt")]
    rprintln!(
        "DMA transfer src: {:010p}, dst: {:010p}, transfer_size: {}",
        src,
        dst,
        transfer_size
    );
    dmac.int_error_clear().write(|w| w.channel0().set_bit());
    dmac.int_tcclear().write(|w| w.channel0().set_bit());
    dmac.channel(0)
        .src_addr()
        .write(|w| unsafe { w.bits(src as usize as u32) });
    dmac.channel(0)
        .dst_addr()
        .write(|w| unsafe { w.bits(dst as usize as u32) });
    dmac.channel(0).control().write(|w| {
        unsafe { w.transfer_size().bits(transfer_size as u16) };
        w.sbsize()._256();
        w.dbsize()._256();
        w.swidth().bits32();
        w.dwidth().bits32();
        w.s().clear_bit();
        w.d().set_bit();
        w.si().set_bit();
        w.di().set_bit();
        w.i().clear_bit()
    });
    dmac.channel(0).configuration().write(|w| {
        w.e().set_bit();
        unsafe { w.src_peripheral().bits(0) };
        unsafe { w.dst_peripheral().bits(0) };
        w.flow_cntrl().mem_to_mem_dma_ctrl();
        w.ie().clear_bit();
        w.itc().clear_bit()
    });
    while dmac.enabled_channels().read().channel0().bit_is_set() {
        core::hint::spin_loop();
    }
}

impl FlashAlgorithm for Algorithm {
    fn new(
        _address: u32,
        _clock: u32,
        #[cfg_attr(not(feature = "dma"), allow(unused))] function: Function,
    ) -> Result<Self, ErrorCode> {
        #[cfg(feature = "rtt")]
        {
            rtt_init_print!();
            rprintln!("Initialize flash algorithm");
        }
        let p = unsafe { pac::Peripherals::steal() };
        p.sys.clk_cntl().modify(|_, w| {
            unsafe { w.sclk_div().bits(0) };
            w.clk_source().hsi()
        });
        p.plic.disable();
        unsafe { riscv::register::mie::write(Mie::from_bits(0)) };
        if function != Function::Verify {
            flash_unlock(&p.flash);
        }
        #[cfg(feature = "dma")]
        if function == Function::Program {
            #[cfg(feature = "rtt")]
            rprintln!("Initialize DMA controller");
            p.sys
                .ahb_clk_en()
                .modify(|_, w| w.ahb_clk_en_dmac0().set_bit());
            p.sys.ahb_rst().modify(|_, w| w.ahb_rst_dmac0().set_bit());
            p.sys.ahb_rst().modify(|_, w| w.ahb_rst_dmac0().clear_bit());
            p.dmac0.dmac_configuration().write(|w| {
                w.e().set_bit();
                w.m1().clear_bit();
                w.m2().clear_bit()
            });
            p.dmac0
                .channel(0)
                .lli()
                .write(|w| unsafe { w.lli().bits(0) });
        }
        #[cfg(feature = "rtt")]
        {
            rprintln!("Flash capacity: {} bytes", flash_capacity_bytes(&p.flash));
            rprint!("Flash unique id: ");
            for data in flash_unique_id(&p.flash) {
                rprint!("{:08x}", data);
            }
            rprintln!();
            rprintln!("Initialized");
        }
        Ok(Self)
    }

    fn erase_all(&mut self) -> Result<(), ErrorCode> {
        #[cfg(feature = "rtt")]
        rprintln!("Erase All");
        let flash = unsafe { Flash::steal() };
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
        #[cfg(feature = "rtt")]
        rprintln!("Erase sector addr: {:#08x}", addr);
        let flash = unsafe { Flash::steal() };
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
        #[cfg(feature = "rtt")]
        rprintln!("Program page addr: {:#08x}, size: {}", addr, data.len());
        let flash = unsafe { Flash::steal() };
        flash.flash_cr().modify(|_, w| {
            w.flash_cr_pg().set_bit();
            w.flash_cr_fastpg().set_bit()
        });
        let words = data.len() / 4;
        let src = unsafe { from_raw_parts(data.as_ptr() as *const u32, words) };
        let dst = unsafe { from_raw_parts_mut(addr as *mut u32, words) };

        #[cfg(feature = "dma")]
        {
            let dmac = unsafe { Dmac0::steal() };
            const TRANSFER_SIZE: usize = 1024;
            let (src_chunks, src_remainder) = src.as_chunks::<TRANSFER_SIZE>();
            let (dst_chunks, dst_remainder) = dst.as_chunks_mut::<TRANSFER_SIZE>();
            for (src_chunk, dst_chunk) in src_chunks.iter().zip(dst_chunks) {
                flash_program_dma(&dmac, src_chunk, dst_chunk);
            }
            if src_remainder.len() > 0 {
                flash_program_dma_raw(
                    &dmac,
                    src_remainder.as_ptr(),
                    dst_remainder.as_mut_ptr(),
                    src_remainder.len(),
                );
            }
        }

        #[cfg(not(feature = "dma"))]
        dst.copy_from_slice(src);

        flash.flash_cr().modify(|_, w| {
            w.flash_cr_pg().clear_bit();
            w.flash_cr_fastpg().clear_bit();
            w.flash_cr_optpg().clear_bit()
        });
        flash_wait_for_busy(&flash);
        Ok(())
    }

    fn verify(&mut self, address: u32, size: u32, data: Option<&[u8]>) -> Result<(), ErrorCode> {
        #[cfg(feature = "rtt")]
        rprintln!("Verify addr: {:#08x}, size: {}", address, size);
        let src = unsafe { from_raw_parts(address as *const u8, size as usize) };
        let count = src
            .iter()
            .zip(data.unwrap())
            .take_while(|(a, b)| **a == **b)
            .count();
        Err(ErrorCode::new(address + count as u32).unwrap())
    }

    #[cfg(feature = "blank-check")]
    fn blank_check(&mut self, address: u32, size: u32, pattern: u8) -> Result<(), ErrorCode> {
        #[cfg(feature = "rtt")]
        rprintln!(
            "Blank check addr: {:#08x}, size: {}, pattern: {}",
            address,
            size,
            pattern
        );
        let dst = unsafe { from_raw_parts(address as *const u8, size as usize) };
        if dst.into_iter().all(|data| *data == pattern) {
            Ok(())
        } else {
            Err(ErrorCode::new(1).unwrap())
        }
    }
}
