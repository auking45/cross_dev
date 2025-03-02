use crate::{
    buildroot::Buildroot,
    config::*,
    env::*,
    error::*,
    linux::Linux,
    opensbi::Opensbi,
    qemu::Qemu,
    ssh::SSH_PORT,
    toolchain::*,
    traits::*,
    utils::{create_dir, get_root_dir, get_work_dir},
};
use std::{collections::HashMap, process::Command};

const DOWNLOAD_DIR: &str = "downloads";
const BUILD_DIR: &str = "builds";
const IMAGE_DIR: &str = "images";

#[derive(Debug)]
pub struct CrossDev {
    config: Config,
    work_dir: String,
    packages: HashMap<PackType, Box<dyn Installable>>,
}

impl CrossDev {
    pub fn new(config: Config) -> Result<Self> {
        let work_dir = format!("{}/{}", get_work_dir()?, config.name);
        set_env(&work_dir)?;

        let mut packages: HashMap<PackType, Box<dyn Installable>> = HashMap::new();
        let mut toolchain_package = None;

        for package in &config.packages {
            // If the package is a toolchain, we will store it in a separate variable
            // since we will need to use it to set the cross toolchain path
            if package.pack_type == PackType::Toolchain {
                toolchain_package = Some(package.clone());
            } else {
                packages.insert(package.pack_type, create_package(&config, package.clone())?);
            }
        }

        if let Some(toolchain_package) = toolchain_package {
            let mut toolchain = Toolchain::new(config.arch.clone(), toolchain_package)?;
            toolchain.setup()?;
            packages.insert(PackType::Toolchain, Box::new(toolchain));
        } else {
            return Err(CrossDevError::NoToolchainInConfig);
        }

        Ok(Self {
            config,
            work_dir,
            packages,
        })
    }

    pub fn work_dir(&self) -> &str {
        &self.work_dir
    }

    pub fn set_work_dir(&mut self, work_dir: String) {
        self.work_dir = work_dir;
    }

    pub fn setup(&mut self) -> Result<()> {
        for package in &mut self.packages {
            if package.0 == &PackType::Toolchain {
                continue;
            }
            package.1.setup()?;
        }

        Ok(())
    }

    pub fn run_qemu(&self, extra_args: Option<Vec<&str>>) -> Result<()> {
        let qemu_pack = self.get_package(PackType::Qemu)?;
        let opensbi_pack = self.get_package(PackType::Opensbi)?;
        let linux_pack = self.get_package(PackType::Linux)?;
        let rootfs_pack = self.get_package(PackType::Buildroot)?;

        let image_dir = get_dir(EnvType::ImageDir)?;

        let qemu_bin = qemu_pack.bin_path();
        let opensbi_bin = format!("{image_dir}/{}", opensbi_pack.bin_name());
        let linux_bin = format!("{image_dir}/{}", linux_pack.bin_name());
        let rootfs_bin = format!("{image_dir}/{}", rootfs_pack.bin_name());

        let qemu_args = format!(
            r#"
            -machine virt
            -nographic
            -smp 4
            -m 2G
            -serial mon:stdio
            -semihosting-config enable=on
            -bios {opensbi_bin}
            -kernel {linux_bin}
            -drive file={rootfs_bin},if=none,format=raw,id=hd0
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

    pub fn get_package(&self, pack_type: PackType) -> Result<&dyn Installable> {
        self.packages
            .get(&pack_type)
            .map(|pkg| pkg.as_ref())
            .ok_or(CrossDevError::PackageError(pack_type))
    }
}

fn set_env(work_dir: &str) -> Result<()> {
    let root_dir = get_root_dir()?;
    let work_dir = work_dir.to_string();
    let download_dir = format!("{work_dir}/{DOWNLOAD_DIR}");
    let build_dir = format!("{work_dir}/{BUILD_DIR}");
    let image_dir = format!("{work_dir}/{IMAGE_DIR}");

    create_dir(&work_dir)?;
    create_dir(&download_dir)?;
    create_dir(&build_dir)?;
    create_dir(&image_dir)?;

    set_env_vars(EnvVars::new(
        root_dir,
        work_dir,
        download_dir,
        build_dir,
        image_dir,
    ))?;

    Ok(())
}

fn create_package(config: &Config, package: Package) -> Result<Box<dyn Installable>> {
    let arch = config.arch.clone();

    match package.pack_type {
        PackType::Toolchain => Ok(Box::new(Toolchain::new(arch, package)?)),
        PackType::Qemu => Ok(Box::new(Qemu::new(arch, package)?)),
        PackType::Opensbi => Ok(Box::new(Opensbi::new(arch, package)?)),
        PackType::Linux => Ok(Box::new(Linux::new(arch, package)?)),
        PackType::Buildroot => Ok(Box::new(Buildroot::new(arch, package)?)),
    }
}
