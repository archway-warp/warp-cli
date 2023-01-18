use std::process::{Command, Stdio};

use crate::{error::WarpError, executable::Executable, utils};
use clap::Args;

#[derive(Args)]
pub struct InitCommand {
    //#[arg(short, long)]
    /// The name of your project's directory
    pub name: String,
}

impl Executable for InitCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let dir = std::env::current_dir()?.join(&self.name);
        println!("Initializing new workspace...");
        let cmd = Command::new("git")
            .arg("clone")
            .arg("https://github.com/secret-warp/warp-template.git")
            .arg(dir.as_os_str())
            .stdout(Stdio::null())
            .spawn()?
            .wait()?;
        if cmd.success() {
            utils::project_config::ProjectConfig::generate_and_save(dir.clone())?;
        } else {
            return Err(WarpError::InitFailed);
        }
        Ok(())
    }
}
