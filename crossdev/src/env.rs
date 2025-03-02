use crate::error::*;
use std::sync::OnceLock;

#[derive(Debug)]
pub enum EnvType {
    RootDir,
    WorkDir,
    DownloadDir,
    BuildDir,
    ImageDir,
}

#[derive(Debug)]
pub struct EnvVars {
    pub root_dir: String,
    pub work_dir: String,
    pub download_dir: String,
    pub build_dir: String,
    pub image_dir: String,
}

impl EnvVars {
    pub fn new(
        root_dir: String,
        work_dir: String,
        download_dir: String,
        build_dir: String,
        image_dir: String,
    ) -> Self {
        Self {
            root_dir,
            work_dir,
            download_dir,
            build_dir,
            image_dir,
        }
    }

    pub fn root_dir(&self) -> &str {
        &self.root_dir
    }

    pub fn work_dir(&self) -> &str {
        &self.work_dir
    }

    pub fn download_dir(&self) -> &str {
        &self.download_dir
    }

    pub fn build_dir(&self) -> &str {
        &self.build_dir
    }

    pub fn image_dir(&self) -> &str {
        &self.image_dir
    }
}

pub static ENV_VARS: OnceLock<EnvVars> = OnceLock::new();

pub fn set_env_vars(env_vars: EnvVars) -> Result<()> {
    ENV_VARS
        .set(env_vars)
        .map_err(|_| CrossDevError::EnvVarsAlreadySet)?;
    Ok(())
}

pub fn get_dir(env_type: EnvType) -> Result<String> {
    let env_vars = ENV_VARS.get().ok_or(CrossDevError::EnvVarsError)?;
    match env_type {
        EnvType::RootDir => Ok(env_vars.root_dir().to_string()),
        EnvType::WorkDir => Ok(env_vars.work_dir().to_string()),
        EnvType::DownloadDir => Ok(env_vars.download_dir().to_string()),
        EnvType::BuildDir => Ok(env_vars.build_dir().to_string()),
        EnvType::ImageDir => Ok(env_vars.image_dir().to_string()),
    }
}
