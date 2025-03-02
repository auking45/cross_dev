use crate::{config::*, env::*, error::Result, ssh::prepare_ssh_key, traits::Installable};
use std::{fs, process::Command};
use xshell::{cmd, Shell};

#[derive(Debug)]
pub struct Buildroot {
    arch: String,
    package: Package,
    build_dir: String,
    buildroot_dir: String,
    br_org_custom_dir: String,
    br_custom_dir: String,
    br_overlay_dir: String,
    bin_name: String,
    bin_path: String,
}

impl Buildroot {
    pub fn new(arch: String, package: Package) -> Result<Self> {
        let root_dir = get_dir(EnvType::RootDir)?;
        let download_dir = get_dir(EnvType::DownloadDir)?;
        let build_dir = get_dir(EnvType::BuildDir)?;
        let buildroot_dir_name = format!("buildroot");
        let build_dir = format!("{build_dir}/{buildroot_dir_name}");
        let buildroot_dir = format!("{download_dir}/{buildroot_dir_name}");
        let br_org_custom_dir = format!("{root_dir}/custom_buildroot");
        let br_custom_dir = format!("{download_dir}/custom_buildroot");
        let br_overlay_dir = format!("{br_custom_dir}/board/riscv/overlay");

        let bin_name = format!("rootfs.ext4");
        let bin_path = format!("{build_dir}/images/{bin_name}");

        Ok(Self {
            arch,
            package,
            build_dir,
            buildroot_dir,
            br_org_custom_dir,
            br_custom_dir,
            br_overlay_dir,
            bin_name,
            bin_path,
        })
    }
}

impl Installable for Buildroot {
    fn name(&self) -> &str {
        &self.package.name
    }

    fn download(&self) -> Result<()> {
        let sh = Shell::new()?;

        match &self.package.download {
            Download::Git(git) => {
                let repo = git.url.as_str();
                let branch = git.branch.as_str();
                let buildroot_dir = self.buildroot_dir.as_str();

                println!("📦 Cloning {repo}...");

                if !sh.path_exists(buildroot_dir) {
                    cmd!(
                        sh,
                        "git clone --depth 1 -b {branch} --single-branch {repo} {buildroot_dir}"
                    )
                    .run_echo()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn build(&self) -> Result<()> {
        let mut sh = Shell::new()?;

        let buildroot_dir = self.buildroot_dir.as_str();
        let buildroot_build_dir = self.build_dir.as_str();
        let br_org_custom_dir = self.br_org_custom_dir.as_str();
        let br_custom_dir = self.br_custom_dir.as_str();
        let download_dir = get_dir(EnvType::DownloadDir)?;
        let br_riscv_config = format!("qemu_riscv64_virt_riscv_defconfig");
        sh.create_dir(buildroot_build_dir)?;
        sh.set_current_dir(buildroot_dir);

        // In order not to copy intermediate files into the original overlay directory
        cmd!(sh, "cp -r {br_org_custom_dir} {download_dir}").run_echo()?;

        // Prepare the ssh key
        prepare_ssh_key(&download_dir)?;

        cmd!(
            sh,
            "make O={buildroot_build_dir} BR2_EXTERNAL={br_custom_dir} {br_riscv_config}"
        )
        .run_echo()?;

        sh.set_current_dir(buildroot_build_dir);

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

        Ok(())
    }

    fn install(&self) -> Result<()> {
        // Copy the binary to the image directory
        let bin_path = self.bin_path.as_str();
        let image_path = format!("{}/{}", get_dir(EnvType::ImageDir)?, self.bin_name);

        println!("📦 Copying {bin_path} to {image_path}...");

        fs::copy(bin_path, image_path)?;

        Ok(())
    }

    fn build_dir(&self) -> &str {
        &self.build_dir
    }

    fn bin_name(&self) -> &str {
        &self.bin_name
    }

    fn bin_path(&self) -> &str {
        &self.bin_path
    }
}
