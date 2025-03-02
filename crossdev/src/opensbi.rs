use crate::{config::*, env::*, error::Result, traits::Installable, utils::*};
use std::fs;
use xshell::{cmd, Shell};

#[derive(Debug)]
pub struct Opensbi {
    arch: String,
    package: Package,
    build_dir: String,
    opensbi_dir: String,
    bin_name: String,
    bin_path: String,
}

impl Opensbi {
    pub fn new(arch: String, package: Package) -> Result<Self> {
        let opensbi_dir_name = format!("opensbi");
        let build_dir = format!("{}/{opensbi_dir_name}", get_dir(EnvType::BuildDir)?);
        let opensbi_dir = format!("{}/{opensbi_dir_name}", get_dir(EnvType::DownloadDir)?);
        let bin_name = format!("fw_jump.bin");
        let bin_path = format!("{build_dir}/platform/generic/firmware/{bin_name}");

        Ok(Self {
            arch,
            package,
            build_dir,
            opensbi_dir,
            bin_name,
            bin_path,
        })
    }
}

impl Installable for Opensbi {
    fn name(&self) -> &str {
        &self.package.name
    }

    fn download(&self) -> Result<()> {
        let sh = Shell::new()?;

        match &self.package.download {
            Download::Git(git) => {
                let repo = git.url.as_str();
                let branch = git.branch.as_str();
                let opensbi_dir = self.opensbi_dir.as_str();

                println!("📦 Cloning {repo}...");

                if !sh.path_exists(opensbi_dir) {
                    cmd!(
                        sh,
                        "git clone --depth 1 -b {branch} --single-branch {repo} {opensbi_dir}"
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

        let cross_toolchain = get_cross_toolchain_path()?;
        let opensbi_dir = self.opensbi_dir.as_str();
        let opensbi_build_dir = self.build_dir.as_str();
        sh.create_dir(opensbi_build_dir)?;
        sh.set_current_dir(opensbi_dir);

        let envs = [("CROSS_COMPILE", &cross_toolchain)];

        for (k, v) in envs {
            sh.set_var(k, v)
        }

        cmd!(
            sh,
            "make PLATFORM=generic O={opensbi_build_dir} -C {opensbi_dir}"
        )
        .run_echo()?;

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
