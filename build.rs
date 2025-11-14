use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut g = TokenStream::new();
    let mut peripherals = vec![
        format_ident!("SYS"),
        format_ident!("PLIC"),
        format_ident!("RTC"),
        format_ident!("FLASH"),
        format_ident!("MAC0"),
        format_ident!("CRC0"),
        format_ident!("USB0"),
        format_ident!("CAN0"),
        format_ident!("WATCHDOG0"),
        format_ident!("FCB0"),
    ];
    peripherals.extend((0..2).map(|x| format_ident!("I2C{x}")));
    peripherals.extend((0..5).map(|x| format_ident!("UART{x}")));
    peripherals.extend((0..5).map(|x| format_ident!("GPTIMER{x}")));
    peripherals.extend((0..2).map(|x| format_ident!("TIMER{x}")));
    peripherals.extend((0..2).map(|x| format_ident!("SPI{x}")));
    peripherals.extend(
        (0..10)
            .cartesian_product(0..8)
            .map(|(x, y)| format_ident!("GPIO{x}_{y}")),
    );
    peripherals.extend((0..8).map(|x| format_ident!("EXTI{x}")));
    peripherals.extend((0..8).map(|x| format_ident!("DMAC0_CH{x}")));
    g.extend(quote! {
        peripherals!(#(#peripherals),*);
    });
    File::create(out_dir.join("_gen.rs"))?.write_all(g.to_string().as_bytes())?;

    println!("cargo::rustc-link-search={}", out_dir.display());
    File::create(out_dir.join("memory.x"))?.write_all(include_bytes!("memory.x"))?;
    println!("cargo::rerun-if-changed=memory.x");
    Ok(())
}
