use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, executable::Executable,
    utils::project_config::ProjectConfig,
};
use clap::{Args, ValueEnum};
use owo_colors::OwoColorize;
use std::path::PathBuf;

#[derive(Args)]
pub struct FrontendCommand {}

impl Executable for FrontendCommand {
    fn execute(
        &self,
        project_root: Option<PathBuf>,
        config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        if project_root.is_none() {
            return Err(WarpError::ProjectFileNotFound);
        };
        let project_root = project_root.unwrap();
        let config = config.unwrap();

        println!(
            "{} {}",
            "Initializing frontend...".bright_yellow(),
            "this may take a moment".yellow()
        );
        profile.init_frontend(&project_root)?;
        Ok(())
    }
}
