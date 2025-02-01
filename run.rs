#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
color-eyre = { version = "0.6" }
---

use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Hello, world!");

    Ok(())
}
