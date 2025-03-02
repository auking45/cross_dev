#!/usr/bin/env -S cargo +nightly -Zscript

---cargo
[dependencies]
clap = { version = "4.5", features = ["derive"] }
color-eyre = { version = "0.6" }
xshell = { version = "0.3.0-pre.2" }
---

use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::Result;
use std::{env, fs::File, io::Write, process::Command, sync::OnceLock};
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
    Ssh,
    Gdb,
    Toolchain,
    Qemu,
    Sbi,
    Linux,
    Buildroot,
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
const OPENSBI_BIN: &str = "fw_jump.bin";
const LINUX_BIN: &str = "Image";
const ROOTFS_BIN: &str = "rootfs.img";

const SSH_PORT: &str = "10025";

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
    pub buildroot_dir: String,
    pub br_org_custom_dir: String,
    pub br_custom_dir: String,
    pub br_overlay_dir: String,
    pub br_riscv_dir: String,
    pub br_riscv_output_dir: String,
    pub br_riscv_config: String,
    pub ssh_dir: String,
    pub ssh_key: String,
    pub gdb_dir: String,
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
        let buildroot_dir = format!("{common_dir}/buildroot");
        let br_org_custom_dir = format!("{root_dir}/custom_buildroot");
        let br_custom_dir = format!("{common_dir}/custom_buildroot");
        let br_overlay_dir = format!("{br_custom_dir}/board/riscv/overlay");
        let br_riscv_dir = format!("{riscv_dir}/buildroot");
        let br_riscv_output_dir = format!("{br_riscv_dir}/output");
        let br_riscv_config = format!("qemu_riscv64_virt_riscv_defconfig");
        let ssh_dir = format!("{br_overlay_dir}/root/.ssh");
        let ssh_key = format!("{ssh_dir}/id_rsa");
        let gdb_dir = format!("{root_dir}/gdb");

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
            buildroot_dir,
            br_org_custom_dir,
            br_custom_dir,
            br_overlay_dir,
            br_riscv_dir,
            br_riscv_output_dir,
            br_riscv_config,
            ssh_dir,
            ssh_key,
            gdb_dir,
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
        "cp -f ./build/platform/generic/firmware/{OPENSBI_BIN} {riscv_image_dir}/"
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
    cmd!(sh, "cp -f ./arch/riscv/boot/{LINUX_BIN} {riscv_image_dir}/").run()?;

    println!("✅ Linux is ready!");

    Ok(())
}

fn build_buildroot() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    sh.set_current_dir(&paths.br_riscv_output_dir);

    // Remove any trailing whitespace from PATH which causes buildroot to fail in
    let clean_path = Command::new("sh")
        .arg("-c")
        .arg("echo $PATH | tr -d ' \t\n'")
        .output()?
        .stdout;
    let clean_path = String::from_utf8(clean_path)?.trim().to_string();
    sh.set_var("PATH", clean_path);

    let nproc = cmd!(sh, "nproc").read()?;
    cmd!(sh, "make -j{nproc}").run_echo()?;

    let image_dir = paths.riscv_images_dir.as_str();
    cmd!(sh, "cp ./images/rootfs.ext2 {image_dir}/{ROOTFS_BIN}").run_echo()?;

    Ok(())
}

fn prepare_buildroot() -> Result<()> {
    let mut sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    println!("🚀 Preparing Buildroot...");

    let repo = "http://github.com/buildroot/buildroot";
    let branch = "2024.11.1";
    let buildroot_dir = paths.buildroot_dir.as_str();

    if !sh.path_exists(buildroot_dir) {
        cmd!(
            sh,
            "git clone --depth 1 -b {branch} --single-branch {repo} {buildroot_dir}"
        )
        .run_echo()?;
    }

    let common_dir = paths.common_dir.as_str();
    let br_org_custom_dir = paths.br_org_custom_dir.as_str();
    let br_custom_dir = paths.br_custom_dir.as_str();
    let br_riscv_output_dir = paths.br_riscv_output_dir.as_str();
    let br_riscv_config = paths.br_riscv_config.as_str();

    sh.create_dir(br_riscv_output_dir)?;
    sh.set_current_dir(&paths.buildroot_dir);

    // In order not to copy intermediate files into the original overlay directory
    cmd!(sh, "cp -r {br_org_custom_dir} {common_dir}").run_echo()?;

    prepare_ssh_key()?;

    cmd!(
        sh,
        "make O={br_riscv_output_dir} BR2_EXTERNAL={br_custom_dir} {br_riscv_config}"
    )
    .run_echo()?;

    build_buildroot()?;

    println!("✅ Buildroot is ready!");

    Ok(())
}

fn prepare_ssh_key() -> Result<()> {
    let sh = Shell::new()?;
    let paths = PATHS.get().unwrap();

    let ssh_dir = paths.ssh_dir.as_str();
    let ssh_key = paths.ssh_key.as_str();
    let ssh_pub = format!("{ssh_key}.pub");
    let authorized_keys = format!("{ssh_dir}/authorized_keys");

    sh.create_dir(ssh_dir)?;

    if sh.path_exists(ssh_key) && sh.path_exists(authorized_keys.as_str()) {
        println!("SSH key already exists");
        return Ok(());
    }

    // Generate SSH key without passphrase
    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("rsa")
        .arg("-N")
        .arg("")
        .arg("-f")
        .arg(ssh_key)
        .arg("-y")
        .status()?;

    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Failed to generate SSH key"));
    }

    // Create the authorized_keys file if it doesn't exist
    if !sh.path_exists(&authorized_keys) {
        File::create(&authorized_keys)?;
    }

    // Append the public key to the authorized_keys file
    let mut file = File::options().append(true).open(&authorized_keys)?;
    let pub_key = std::fs::read_to_string(&ssh_pub)?;
    writeln!(file, "{}", pub_key)?;

    println!("✅ SSH key is ready!");

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
    prepare_buildroot()?;

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
        -bios {image_dir}/{OPENSBI_BIN}
        -kernel {image_dir}/{LINUX_BIN}
        -drive file={image_dir}/{ROOTFS_BIN},if=none,format=raw,id=hd0
        -device virtio-blk-device,drive=hd0
        -netdev user,id=net0,hostfwd=tcp::{SSH_PORT}-:22
        -device virtio-net-device,netdev=net0
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

fn run_ssh() -> Result<()> {
    let paths = PATHS.get().unwrap();

    let ssh_bin = "ssh";
    let ssh_key = paths.ssh_key.as_str();
    let ssh_args = [
        "-i",
        ssh_key,
        "root@localhost",
        "-p",
        SSH_PORT,
        "-o",
        "StrictHostKeyChecking no",
    ];

    let mut child = Command::new(ssh_bin)
        .args(ssh_args)
        .spawn()
        .expect("Failed to connect via ssh: {ssh_bin} {ssh_args:?}");

    let _ = child.wait().unwrap();

    Ok(())
}

fn run_gdb() -> Result<()> {
    let paths = PATHS.get().unwrap();

    let gdb_dir = paths.gdb_dir.as_str();
    let _ = env::set_current_dir(gdb_dir);

    let gdb_bin = "gdb-multiarch";
    let gdb_args = format!(
        r#"
        -x {gdb_dir}/.gdbinit
        --cd={gdb_dir}
        "#,
    );
    let gdb_args: Vec<_> = gdb_args.split_whitespace().collect();

    let mut child = Command::new(gdb_bin)
        .args(gdb_args)
        .env("GDB_WORK_DIR", paths.work_dir.clone())
        .spawn()
        .expect("Failed to launch target: {gdb_bin} {gdb_args}");

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
        Some(Commands::Ssh) => {
            run_ssh()?;
        }
        Some(Commands::Gdb) => {
            run_gdb()?;
        }
        Some(Commands::Toolchain) => {
            prepare_toolchain()?;
        }
        Some(Commands::Qemu) => {
            prepare_qemu()?;
        }
        Some(Commands::Sbi) => {
            prepare_opensbi()?;
        }
        Some(Commands::Linux) => {
            prepare_linux()?;
        }
        Some(Commands::Buildroot) => {
            prepare_buildroot()?;
        }
        None => {
            println!("No command provided");
        }
    }

    Ok(())
}
