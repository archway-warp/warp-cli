use std::path::PathBuf;

pub use clap::{arg, Args};
use clap::{Subcommand, ValueEnum};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, executable::Executable,
    utils::project_config::ProjectConfig,
};

#[derive(Args)]
pub struct ConfigCommand {
    /// Configuration subcommand
    #[command(subcommand)]
    subcommand: SetSubcommand,
}

#[derive(Subcommand)]
enum SetSubcommand {
    /// Set the config parameter
    Set(ConfigArgs),
    /// Get the current value of the parameter
    Get(ConfigArgs),
}

#[derive(Args, Clone)]
pub struct ConfigArgs {
    /// Contract optimization backend
    #[arg(short, long, value_enum)]
    optimizer_backend: Option<OptimizerBackend>,
    /// Quickly switch between different networks
    #[arg(short, long, value_enum)]
    network: Option<NetworkConfig>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OptimizerBackend {
    /// Use CosmWasm's rust-optimizer - specifically, the workspace-optimizer docker image
    Default,
    /// Use 'cw-optimizoor' which doesn't require docker (it needs to be installed and in $PATH)
    CwOptimizoor,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum NetworkConfig {
    /// The long-awaited Archway Mainnet
    Mainnet,
    /// The official supported Archway Testnet
    Testnet,
    /// Local Archway Testnet (WIP)
    Local,
}

impl Executable for ConfigCommand {
    fn execute(
        &self,
        project_root: Option<PathBuf>,
        config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        if project_root.is_none() {
            return Err(WarpError::ProjectFileNotFound);
        };
        let mut config = config.unwrap().clone();

        let modify_values: bool;
        let args = match &self.subcommand {
            SetSubcommand::Set(x) => {
                modify_values = true;
                x
            }
            SetSubcommand::Get(x) => {
                modify_values = false;
                x
            }
        };

        if let Some(x) = &args.optimizer_backend {
            if modify_values {
                config.tooling.optimizer_backend = match x {
                    OptimizerBackend::Default => "default",
                    OptimizerBackend::CwOptimizoor => "cw-optimizoor",
                }
                .to_owned();
            }
            println!(
                " {} {}: {}",
                "=>".bright_yellow(),
                "Optimizer Backend".bright_blue(),
                config.tooling.optimizer_backend.bright_green()
            );
        }

        // Network Config
        if let Some(x) = &args.network {
            if modify_values {
                let params = profile.network_params(&x);
                config.network = params;
            }
            println!(
                "{} {}: {}",
                "=>".bright_yellow(),
                "Network Configuration".bright_blue(),
                serde_json::to_string(&x)?
            );
        }

        if modify_values {
            config.save_project_config()?;
        }
        Ok(())
    }
}
