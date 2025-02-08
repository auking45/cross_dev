#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[dependencies]
clap = { version = "4.5", features = ["derive"] }
color-eyre = { version = "0.6" }
xshell = { version = "0.3.0-pre.2" }
---

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::Result;
use std::{env, process::Command, sync::OnceLock};
use xshell::{cmd, Shell};

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
    Setup(SubArgs),
    Run(RunCmd),
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

const QEMU_VERSION: &str = "9.2.0";

#[derive(Debug)]
struct Paths {
    pub root_dir: String,
    pub work_dir: String,
    pub common_dir: String,
    pub toolchain_dir: String,
    pub riscv_toolchain_dir: String,
    pub riscv_cross_toolchain: String,
    pub riscv_dir: String,
    pub riscv_images_dir: String,
    pub qemu_dir: String,
    pub qemu_build_dir: String,
    pub qemu_bin: String,
    pub opensbi_dir: String,
    pub linux_dir: String,
    pub linux_build_dir: String,
}

impl Paths {
    pub fn new() -> Self {
        let sh = Shell::new().unwrap();
        let root_dir = cmd!(sh, "git rev-parse --show-toplevel").read().unwrap();
        let work_dir = format!("{root_dir}/.work_rs");
        let common_dir = format!("{work_dir}/common");
        let toolchain_dir = format!("{common_dir}/toolchain");
        let riscv_toolchain_dir = format!("{toolchain_dir}/riscv/bin");
        let riscv_cross_toolchain = format!("{riscv_toolchain_dir}/riscv64-unknown-linux-gnu-");
        let riscv_dir = format!("{work_dir}/riscv");
        let riscv_images_dir = format!("{riscv_dir}/images");
        let qemu_dir = format!("{common_dir}/qemu-{QEMU_VERSION}");
        let qemu_build_dir = format!("{riscv_dir}/qemu/build");
        let qemu_bin = format!("{qemu_build_dir}/qemu-system-riscv64");
        let opensbi_dir = format!("{riscv_dir}/opensbi");
        let linux_dir = format!("{common_dir}/linux");
        let linux_build_dir = format!("{riscv_dir}/linux/build");

        Self {
            root_dir,
            work_dir,
            common_dir,
            toolchain_dir,
            riscv_toolchain_dir,
            riscv_cross_toolchain,
            riscv_dir,
            riscv_images_dir,
            qemu_dir,
            qemu_build_dir,
            qemu_bin,
            opensbi_dir,
            linux_dir,
            linux_build_dir,
        }
    }
}

static PATHS: OnceLock<Paths> = OnceLock::new();

fn prepare_toolchain() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    println!("🚀 Preparing toolchains...");

    let ubuntu_ver = cmd!(sh, "lsb_release -rs").read()?;
    let tag = "2025.01.20";
    let dn_url = "https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download";
    let filename = format!("riscv64-glibc-ubuntu-{ubuntu_ver}-llvm-nightly-{tag}-nightly.tar.xz");
    let full_url = format!("{dn_url}/{tag}/{filename}");
    let toolchain_dir = paths.toolchain_dir.as_str();

    if !sh.path_exists(paths.riscv_toolchain_dir.as_str()) {
        sh.create_dir(toolchain_dir)?;
        sh.set_current_dir(toolchain_dir);

        cmd!(sh, "wget {full_url}").run_echo()?;
        cmd!(sh, "tar -xf {filename}").run_echo()?;
        cmd!(sh, "rm -f {filename}").run_echo()?;
    };

    println!("✅ Toolchains are ready!");

    Ok(())
}

fn prepare_qemu() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    println!("🚀 Preparing QEMU...");

    let qemu_dir = paths.qemu_dir.as_str();
    let qemu_bin = paths.qemu_bin.as_str();
    let qemu_build_dir = paths.qemu_build_dir.as_str();

    if !sh.path_exists(qemu_dir) {
        sh.set_current_dir(paths.common_dir.as_str());

        let qemu_url = format!("https://download.qemu.org/qemu-{QEMU_VERSION}.tar.xz");
        cmd!(sh, "wget {qemu_url}").run_echo()?;
        cmd!(sh, "tar -xf qemu-{QEMU_VERSION}.tar.xz").run_echo()?;
        cmd!(sh, "rm -f qemu-{QEMU_VERSION}.tar.xz").run_echo()?;
    }

    if !sh.path_exists(qemu_bin) {
        sh.create_dir(qemu_build_dir)?;
        sh.set_current_dir(qemu_build_dir);

        cmd!(sh, "{qemu_dir}/configure --target-list=riscv64-softmmu").run_echo()?;
        let nproc = cmd!(sh, "nproc").read()?;
        cmd!(sh, "make -j{nproc}").run_echo()?;
    }

    cmd!(sh, "{qemu_bin} --version").run_echo()?;

    println!("✅ QEMU is ready!");

    Ok(())
}

fn prepare_opensbi() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    println!("🚀 Preparing OpenSBI...");

    let repo = "https://github.com/riscv-software-src/opensbi.git";
    let branch = "v1.6";
    let opensbi_dir = paths.opensbi_dir.as_str();

    if !sh.path_exists(opensbi_dir) {
        cmd!(sh, "git clone -b {branch} {repo} {opensbi_dir}").run_echo()?;
    }

    let envs = [("CROSS_COMPILE", &paths.riscv_cross_toolchain)];

    for (k, v) in envs {
        sh.set_var(k, v);
    }

    sh.set_current_dir(opensbi_dir);

    cmd!(sh, "make PLATFORM=generic").run_echo()?;

    let riscv_image_dir = paths.riscv_images_dir.as_str();
    cmd!(
        sh,
        "cp -f ./build/platform/generic/firmware/fw_jump.bin {riscv_image_dir}/"
    )
    .run()?;

    println!("✅ OpenSBI is ready!");

    Ok(())
}

fn prepare_linux() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    println!("🚀 Preparing Linux...");

    let repo = "https://github.com/Rust-for-Linux/linux.git";
    let branch = "rust-next";
    let linux_dir = paths.linux_dir.as_str();
    let linux_build_dir = paths.linux_build_dir.as_str();

    if !sh.path_exists(linux_dir) {
        cmd!(
            sh,
            "git clone --depth 1 -b {branch} --single-branch {repo} {linux_dir}"
        )
        .run_echo()?;
    }

    sh.create_dir(linux_build_dir)?;
    sh.set_current_dir(linux_build_dir);

    let envs = [
        ("ARCH", "riscv"),
        ("CROSS_COMPILE", &paths.riscv_cross_toolchain),
    ];

    for (k, v) in envs {
        sh.set_var(k, v)
    }

    cmd!(sh, "make O={linux_build_dir} -C {linux_dir} defconfig").run_echo()?;

    // if there are extra configs
    let script_dir = paths.root_dir.as_str();
    cmd!(sh, "{linux_dir}/scripts/kconfig/merge_config.sh .config {script_dir}/configs/linux/extra.config")
        .run_echo()?;
    // Sample command to enable a config
    // cmd!(sh, "{linux_dir}/scripts/config --file .config --enable CONFIG_XXXXXX").run_echo()?;

    let nproc = cmd!(sh, "nproc").read()?;
    cmd!(sh, "make -j{nproc}").run_echo()?;

    let riscv_image_dir = paths.riscv_images_dir.as_str();
    cmd!(sh, "cp -f ./arch/riscv/boot/Image {riscv_image_dir}/").run()?;

    println!("✅ Linux is ready!");

    Ok(())
}

fn setup() -> Result<()> {
    let sh = Shell::new()?;

    let paths = PATHS.get().unwrap();

    println!("Working directory: {}", paths.work_dir);

    sh.create_dir(&paths.work_dir)?;
    sh.create_dir(&paths.common_dir)?;
    sh.create_dir(&paths.riscv_images_dir)?;

    prepare_toolchain()?;
    prepare_qemu()?;
    prepare_opensbi()?;
    prepare_linux()?;

    Ok(())
}

fn run_qemu(extra_args: Option<Vec<&str>>) -> Result<()> {
    let paths = PATHS.get().unwrap();

    let _ = env::set_current_dir(&paths.qemu_build_dir);

    let qemu_bin = paths.qemu_bin.as_str();
    let image_dir = paths.riscv_images_dir.as_str();
    let qemu_args = format!(
        r#"
        -machine virt
        -nographic
        -smp 4
        -m 2G
        -serial mon:stdio
        -semihosting-config enable=on
        -bios {image_dir}/fw_jump.bin
        -kernel {image_dir}/Image
        "#
    );

    let mut qemu_args: Vec<_> = qemu_args.split_whitespace().collect();
    qemu_args.push("-append");
    qemu_args.push("console=ttyS0 ro root=/dev/vda init=/sbin/init");

    if let Some(extra) = extra_args {
        qemu_args.extend(extra);
    }

    println!("{qemu_args:?}");

    let mut child = Command::new(qemu_bin)
        .args(qemu_args)
        .spawn()
        .expect("Failed to launch target: {qemu_bin} {qemu_args}");

    let _ = child.wait()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let paths = Paths::new();

    PATHS.set(paths).unwrap();

    match &cli.command {
        Some(Commands::Setup(_)) => {
            setup()?;
        }
        Some(Commands::Run(runcmd)) => {
            let extra_args = runcmd.debug.then(|| vec!["-s", "-S"]);
            run_qemu(extra_args)?;
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}
