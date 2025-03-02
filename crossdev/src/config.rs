use color_eyre::eyre::Result;
use core::fmt;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub arch: String,
    pub build_type: String,
    pub packages: Vec<Package>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Package {
    pub name: String,
    pub pack_type: PackType,
    pub version: String,
    #[serde(flatten)]
    pub download_type: DownloadType,
    pub download: Download,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "download_type", rename_all = "lowercase")]
pub enum DownloadType {
    Git,
    File,
    Apt,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Download {
    Git(GitDownload),
    File(FileDownload),
    Apt(AptDownload),
}

#[derive(Clone, Debug, Deserialize)]
pub struct GitDownload {
    pub url: String,
    pub branch: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FileDownload {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AptDownload {
    pub package_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackType {
    Toolchain,
    Qemu,
    Opensbi,
    Linux,
    Buildroot,
}

impl fmt::Display for PackType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn read_config_from_file<P>(path: P) -> Result<Config>
where
    P: AsRef<Path> + std::fmt::Debug,
{
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}
