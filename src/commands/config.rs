use std::process::Command;

pub use clap::{arg, Args};
use clap::{Subcommand, ValueEnum};

use crate::{error::WarpError, executable::Executable, utils::project_config::ProjectConfig};

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
    /// Set a contract optimization backend
    #[arg(short, long, value_enum)]
    optimizer_backend: Option<OptimizerBackend>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OptimizerBackend {
    /// Use CosmWasm's rust-optimizer - specifically, the workspace-optimizer docker image
    Default,
    /// Use 'cw-optimizoor' which doesn't require docker (it needs to be installed and in $PATH)
    CwOptimizoor,
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
                " => Optimizer Backend: {}",
                &config.tooling.optimizer_backend
            );
        }

        if modify_values {
            config.save_project_config()?;
        }
        Ok(())
    }
}
