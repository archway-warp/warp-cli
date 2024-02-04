use std::path::PathBuf;

use crate::{
    chains::chain_profile::ChainProfile,
    error::WarpError,
    executable::Executable,
    utils::{self, project_config::ProjectConfig},
};
use clap::Args;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct InitCommand {
    //#[arg(short, long)]
    /// The name of your project's directory
    pub name: String,
}

impl Executable for InitCommand {
    fn execute(
        &self,
        _project_root: Option<PathBuf>,
        _config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        let dir = std::env::current_dir()?.join(&self.name);
        println!("{}", "Initializing new workspace...".bright_yellow());
        profile.init_project(&dir)?;
        Ok(())
    }
}
