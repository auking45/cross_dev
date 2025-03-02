use clap::{Args, Parser, Subcommand};
use color_eyre::Result;
use crossdev::{config::*, crossdev::*, env::*, gdb::*, ssh::*, utils::*};
use std::fs;

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
    Config(ConfigCmd),
    Setup(SubArgs),
    Run(RunCmd),
    Ssh,
    Gdb,
    Toolchain,
    Qemu,
    Sbi,
    Linux,
    Buildroot,
}

#[derive(Args, Debug)]
struct ConfigCmd {
    #[arg(value_name = "FILE")]
    file: std::path::PathBuf,
}

#[derive(Args, Debug)]
struct RunCmd {
    #[arg(short, long)]
    debug: bool,
}

#[derive(Args, Debug)]
struct SubArgs {
    name: Option<String>,
}

const CONFIG_FILE_NAME: &str = ".config.toml";
const DEFAULT_CONFIG_PATH: &str = "configs/default.toml";

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let root_dir = get_root_dir()?;
    let work_dir = get_work_dir()?;
    let config_path = format!("{work_dir}/{CONFIG_FILE_NAME}");
    let def_config_path = format!("{root_dir}/{DEFAULT_CONFIG_PATH}");

    if let Some(Commands::Config(cfg)) = &cli.command {
        let filepath = cfg.file.to_str().unwrap();
        println!("📋 Copying {filepath} to {config_path}");
        fs::copy(filepath, &config_path)?;
        return Ok(());
    }

    check_config_file(&config_path, &def_config_path)?;

    let config = read_config_from_file(&config_path)?;
    let mut xdev = CrossDev::new(config)?;

    match &cli.command {
        Some(Commands::Config(cfg)) => {
            panic!("Unreachable because it's handled above");
        }
        Some(Commands::Setup(_)) => {
            xdev.setup()?;
        }
        Some(Commands::Run(runcmd)) => {
            let extra_args = runcmd.debug.then(|| vec!["-s", "-S"]);
            xdev.run_qemu(extra_args)?;
        }
        Some(Commands::Ssh) => {
            let download_dir = get_dir(EnvType::DownloadDir)?;
            run_ssh(&download_dir)?;
        }
        Some(Commands::Gdb) => {
            run_gdb()?;
        }
        Some(Commands::Toolchain) => {
            let package = xdev.get_package(PackType::Toolchain)?;
            package.setup()?;
        }
        Some(Commands::Qemu) => {
            let package = xdev.get_package(PackType::Qemu)?;
            package.setup()?;
        }
        Some(Commands::Sbi) => {
            let package = xdev.get_package(PackType::Opensbi)?;
            package.setup()?;
        }
        Some(Commands::Linux) => {
            let package = xdev.get_package(PackType::Linux)?;
            package.setup()?;
        }
        Some(Commands::Buildroot) => {
            let package = xdev.get_package(PackType::Buildroot)?;
            package.setup()?;
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}
