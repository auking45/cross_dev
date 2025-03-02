use crate::error::Result;
use std::fmt::Debug;

pub trait Installable: Debug {
    fn name(&self) -> &str;
    fn download(&self) -> Result<()>;
    fn build(&self) -> Result<()>;
    fn install(&self) -> Result<()>;

    fn build_dir(&self) -> &str;
    fn bin_name(&self) -> &str;
    fn bin_path(&self) -> &str;

    fn setup(&self) -> Result<()> {
        println!("🚀 Preparing package: {}", self.name());

        self.download()?;

        println!("🔧 Building {}", self.name());
        self.build()?;
        println!("✨ {} build complete!", self.name());

        println!("🚚 Installing {}", self.name());
        self.install()?;
        println!("✨ {} installation complete!", self.name());

        println!("✅ Package {} prepared successfully", self.name());

        Ok(())
    }
}
