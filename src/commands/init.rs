use std::path::PathBuf;

use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, executable::Executable,
    utils::project_config::ProjectConfig,
};
use clap::{Args, ValueEnum};
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct InitCommand {
    //#[arg(short, long)]
    /// The name of your project's directory
    pub name: String,
    #[arg(short, long, value_enum)]
    chain: ChainParam,
}

#[derive(ValueEnum, Clone)]
pub enum ChainParam {
    Archway,
    Xion,
}

impl ChainParam {
    pub fn get_chain_profile(&self) -> Box<dyn ChainProfile> {
        match self {
            ChainParam::Archway => Box::new(crate::chains::archway::ArchwayProfile),
            ChainParam::Xion => Box::new(crate::chains::xion::XionProfile),
        }
    }
}

impl Executable for InitCommand {
    fn execute(
        &self,
        _project_root: Option<PathBuf>,
        _config: Option<ProjectConfig>,
        _profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        let profile = &self.chain.get_chain_profile();
        let dir = std::env::current_dir()?.join(&self.name);
        println!("{}", "Initializing new workspace...".bright_yellow());
        profile.init_project(&dir)?;
        ProjectConfig::generate_and_save(
            dir,
            profile.network_params(&crate::commands::config::NetworkConfig::Testnet),
        )?;
        Ok(())
    }
}
