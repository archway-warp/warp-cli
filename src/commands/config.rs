use std::process::Command;

pub use clap::{arg, Args};
use clap::{Subcommand, ValueEnum};
use owo_colors::OwoColorize;

use crate::{
    error::WarpError,
    executable::Executable,
    utils::project_config::{Network, ProjectConfig},
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum NetworkConfig {
    /// The official supported Archway Testnet
    Constantine,
    /// Experimental Archway Testnet
    Titus,
    /// Local Archway Testnet (WIP)
    Local,
}

impl Executable for ConfigCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let (root, mut config) = ProjectConfig::parse_project_config()?;
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
        if let Some(x) = &args.network {}

        if modify_values {
            config.save_project_config()?;
        }
        Ok(())
    }
}

impl NetworkConfig {
    fn network_params(&self) -> Network {
        match self {
            NetworkConfig::Constantine => Network {
                chain_id: "constantine-3".to_owned(),
                rpc_url: "https://rpc.constantine.archway.tech:443".to_owned(),
                denom: "aconst".to_owned(),
            },
            NetworkConfig::Titus => Network {
                chain_id: "titus-1".to_owned(),
                rpc_url: " https://rpc.titus-1.archway.tech:443".to_owned(),
                denom: "".to_owned(),
            },
            NetworkConfig::Local => todo!(),
        }
    }
}
