use crate::{config::*, env::*, error::*, traits::Installable, utils::*};
use xshell::{cmd, Shell};

const TOOLCHAIN_DIR: &str = "toolchains";

#[derive(Debug)]
pub struct Toolchain {
    arch: String,
    package: Package,
    toolchain_dir: String,
}

impl Toolchain {
    pub fn new(arch: String, package: Package) -> Result<Self> {
        let toolchain_dir = format!("{}/{}", get_dir(EnvType::DownloadDir)?, TOOLCHAIN_DIR);

        Ok(Self {
            arch,
            toolchain_dir,
            package,
        })
    }

    fn find_toolchain(&self) -> Result<String> {
        let sh = Shell::new()?;

        let arch_name = &self.arch;
        let find_str = format!("*{arch_name}*-gcc");
        let toolchain_dir = self.toolchain_dir.as_str();

        // Ensure the toolchain directory exists
        if !sh.path_exists(toolchain_dir) {
            sh.create_dir(toolchain_dir)?;
        }

        let toolchain = cmd!(sh, "find {toolchain_dir} -name {find_str}").read()?;

        Ok(toolchain)
    }
}

impl Installable for Toolchain {
    fn name(&self) -> &str {
        &self.package.name
    }

    fn download(&self) -> Result<()> {
        match &self.package.download {
            Download::File(download) => {
                let url = &download.url;
                let filename = url.split('/').last().unwrap();
                let download_dir = get_dir(EnvType::DownloadDir)?;
                let toolchain_dir = self.toolchain_dir.as_str();

                let mut sh = Shell::new()?;

                let toolchain = self.find_toolchain()?;
                if toolchain.is_empty() {
                    sh.create_dir(toolchain_dir)?;
                    sh.set_current_dir(download_dir.as_str());

                    println!("📦 Downloading {url}...");

                    cmd!(sh, "wget -O {filename} {url}").run_echo()?;
                    cmd!(sh, "tar -xf {filename} -C {toolchain_dir}").run_echo()?;
                    cmd!(sh, "rm -f {filename}").run_echo()?;
                }
            }
            Download::Apt(download) => {
                let package_name = &download.package_name;
                println!("📦 Installing {package_name} via APT...");
                let sh = Shell::new()?;
                cmd!(sh, "sudo apt update").run_echo()?;
                cmd!(sh, "sudo apt install -y {package_name}").run_echo()?;
            }
            Download::Git(download) => {
                let url = &download.url;
                let branch = &download.branch;
                println!("📦 Cloning {url} (branch: {branch})...");
                let sh = Shell::new()?;
                cmd!(sh, "git clone --branch {branch} {url}").run_echo()?;
            }
        }

        Ok(())
    }

    fn build(&self) -> Result<()> {
        let sh = Shell::new()?;

        let toolchain = self.find_toolchain()?;
        if toolchain.is_empty() {
            println!("❌ Toolchain not found!");
            return Err(CrossDevError::GccNotFound);
        }

        set_cross_toolchain_path(toolchain.replace("gcc", ""))?;

        // Test toolchain
        let cross_toolchain = get_cross_toolchain_path()?;
        cmd!(sh, "{cross_toolchain}gcc --version").run_echo()?;

        Ok(())
    }

    fn install(&self) -> Result<()> {
        Ok(())
    }

    fn build_dir(&self) -> &str {
        "".into()
    }

    fn bin_name(&self) -> &str {
        "".into()
    }

    fn bin_path(&self) -> &str {
        "".into()
    }
}
