use crate::{config::*, env::*, error::Result, traits::Installable};
use xshell::{cmd, Shell};

#[derive(Debug)]
pub struct Qemu {
    arch: String,
    package: Package,
    build_dir: String,
    qemu_dir: String,
    bin_name: String,
    bin_path: String,
}

impl Qemu {
    pub fn new(arch: String, package: Package) -> Result<Self> {
        let qemu_dir_name = match &package.download {
            Download::File(download) => {
                let url = &download.url;
                let filename = url.split('/').last().unwrap();
                let filename_without_ext = filename.rsplitn(3, '.').last().unwrap();
                filename_without_ext.to_string()
            }
            Download::Git(git) => {
                let url = &git.url;
                let filename = url.split('/').last().unwrap();
                filename.split('.').next().unwrap().to_string()
            }
            _ => "".to_string(),
        };

        let build_dir = format!("{}/{qemu_dir_name}", get_dir(EnvType::BuildDir)?);
        let qemu_dir = format!("{}/{qemu_dir_name}", get_dir(EnvType::DownloadDir)?);
        let bin_name = format!("qemu-system-{arch}");
        let bin_path = format!("{build_dir}/{bin_name}");

        Ok(Self {
            arch,
            package,
            build_dir,
            qemu_dir,
            bin_name,
            bin_path,
        })
    }
}

impl Installable for Qemu {
    fn name(&self) -> &str {
        &self.package.name
    }

    fn download(&self) -> Result<()> {
        match &self.package.download {
            Download::File(download) => {
                let url = &download.url;
                let filename = url.split('/').last().unwrap();
                let qemu_dir = &self.qemu_dir;

                let mut sh = Shell::new()?;

                if !sh.path_exists(qemu_dir) {
                    sh.set_current_dir(get_dir(EnvType::DownloadDir)?);

                    println!("📦 Downloading {url}...");

                    cmd!(sh, "wget {url}").run_echo()?;
                    cmd!(sh, "tar -xf {filename}").run_echo()?;
                    cmd!(sh, "rm -f {filename}").run_echo()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn build(&self) -> Result<()> {
        let mut sh = Shell::new()?;

        let qemu_dir = self.qemu_dir.as_str();
        let qemu_build_dir = self.build_dir.as_str();
        let bin_path = self.bin_path.as_str();

        if !sh.path_exists(bin_path) {
            sh.create_dir(qemu_build_dir)?;
            sh.set_current_dir(qemu_build_dir);

            cmd!(sh, "{qemu_dir}/configure --target-list=riscv64-softmmu").run_echo()?;
            let nproc = cmd!(sh, "nproc").read()?;
            cmd!(sh, "make -j{nproc}").run_echo()?;
        }

        cmd!(sh, "{bin_path} --version").run_echo()?;

        Ok(())
    }

    fn install(&self) -> Result<()> {
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
