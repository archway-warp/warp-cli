use clap::Subcommand;
pub use clap::{arg, Args};

use crate::{
    error::WarpError,
    executable::Executable,
    utils::{deployment_result::DeploymentResult, project_config::ProjectConfig, secretcli_util},
};

#[derive(Args)]
pub struct WasmCommand {
    /// Wasm Subcommand
    #[command(subcommand)]
    subcommand: WasmSubcommand,
}

#[derive(Subcommand)]
pub enum WasmSubcommand {
    /// Execute a contract in the workspace
    Execute(WasmExecuteArgs),
    ///Query a contract in the workspace
    Query(WasmQueryArgs),
}

#[derive(Args, Clone)]
pub struct WasmExecuteArgs {
    /// Contract ID (from the Warp.toml file)
    #[arg(required = true)]
    pub contract: String,
    /// JSON-serialized contract arguments
    #[arg(required = true)]
    pub arguments: String,
    #[arg(long, short)]
    pub from: Option<String>,
    #[arg(long)]
    pub funds: Option<String>, // TODO: Implement
    #[arg(long, short)]
    pub yes: Option<bool>, // TODO: Implement
}

#[derive(Args, Clone)]
pub struct WasmQueryArgs {
    /// Contract ID (from the Warp.toml file)
    #[arg(required = true)]
    pub contract: String,
    /// JSON-serialized contract arguments
    #[arg(required = true)]
    pub arguments: String,
}

impl Executable for WasmCommand {
    fn execute(&self) -> Result<(), crate::error::WarpError> {
        DeploymentResult::exists()?;
        let (_, config) = ProjectConfig::parse_project_config()?;
        let (_, mut deployments) = DeploymentResult::parse()?;

        // Translate contract address
        let contract_id = match &self.subcommand {
            WasmSubcommand::Execute(x) => &x.contract,
            WasmSubcommand::Query(x) => &x.contract,
        };
        let contract_address = deployments
            .network(&config.network.chain_id)
            .get(contract_id);
        if contract_address.is_none() {
            return Err(WarpError::ContractIdNotFound(contract_id.to_owned()));
        }
        let contract_address = contract_address.unwrap();

        match &self.subcommand {
            WasmSubcommand::Execute(x) => {
                let from = x.from.as_ref().unwrap_or(&config.autodeploy.account_id);
                if from.is_empty() {
                    return Err(WarpError::UnspecifiedWallet);
                }

                let password =
                    rpassword::prompt_password("Enter your keyring password (if using/needed):")?;
                let password = if password.is_empty() {
                    None
                } else {
                    Some(password.as_str())
                };

                secretcli_util::execute_contract(
                    &contract_address,
                    &x.arguments,
                    &from,
                    password,
                    &config,
                )?;
            }
            WasmSubcommand::Query(x) => {
                let result =
                    secretcli_util::query_contract_smart(&contract_address, &x.arguments, &config)?;
                println!("{result}");
            }
        }
        Ok(())
    }
}
