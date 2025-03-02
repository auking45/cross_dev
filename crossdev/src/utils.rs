use crate::error::*;
use std::{env, fs, path::Path, sync::OnceLock};
use xshell::{cmd, Shell};

const DEFAULT_ROOT_DIR: &str = ".crossdev";

static CROSS_TOOLCHAIN_PATH: OnceLock<String> = OnceLock::new();

pub fn get_root_dir() -> Result<String> {
    let sh = Shell::new()?;
    let output = cmd!(sh, "git rev-parse --show-toplevel").read()?;

    Ok(output.trim().to_string())
}

pub fn get_work_dir() -> Result<String> {
    let home_dir = env::var("HOME")?;
    let default_work_dir = format!("{}/{}", home_dir, DEFAULT_ROOT_DIR);
    let work_dir = env::var("CROSSDEV_ROOT_DIR").unwrap_or(default_work_dir);

    Ok(work_dir)
}

pub fn check_config_file(config: &str, default_config: &str) -> Result<()> {
    let config_file = Path::new(config);
    let default_config_file = Path::new(default_config);

    if !config_file.exists() {
        fs::copy(default_config_file, config_file)?;
    }

    Ok(())
}

pub fn set_cross_toolchain_path(path: String) -> Result<()> {
    CROSS_TOOLCHAIN_PATH
        .set(path)
        .map_err(|_| CrossDevError::CrossToolchainPathAlreadySet)?;
    Ok(())
}

pub fn get_cross_toolchain_path() -> Result<&'static str> {
    CROSS_TOOLCHAIN_PATH
        .get()
        .map(|s| s.as_str())
        .ok_or(CrossDevError::CrossToolchainPathNotSet)
}

pub fn create_dir(dir: &str) -> Result<()> {
    let path = Path::new(dir);

    if !path.exists() {
        println!("Creating directory: {}", dir);
        fs::create_dir_all(path)?;
    } else {
        println!("Directory already exists: {}", dir);
    }

    // Verify the directory was created
    if !path.exists() {
        return Err(CrossDevError::DirectoryNotCreated(dir.to_string()));
    }

    Ok(())
}
