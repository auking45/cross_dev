#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[dependencies]
clap = { version = "4.5", features = ["derive"] }
color-eyre = { version = "0.6" }
---

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

/// Cross-platform setup script
#[derive(Parser, Debug)]
#[command(name = "Cross-platform setup script")]
#[command(author = "Jinha Hwang, auking45@gmail.com")]
#[command(version = "0.1.0")]
#[command(about = "A script to setup a cross development environment")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Setup,
}

fn setup() -> Result<()> {
    println!("Setting up a new environment");
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Setup) => {
            setup()?;
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}
