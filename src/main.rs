mod archway;
pub mod chains;
mod commands;
mod consts;
mod error;
mod executable;
mod utils;

use chains::{archway::ArchwayProfile, chain_profile::ChainProfile};
use clap::{command, Parser, Subcommand};
use commands::{
    autodeploy::AutoDeployCommand, build::BuildCommand, config::ConfigCommand, init::InitCommand,
    new::NewCommand, node::NodeCommand, test::TestCommand, wasm::WasmCommand,
};
use error::WarpError;
use executable::Executable;
use owo_colors::OwoColorize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Warp project
    Init(InitCommand),
    /// Configure the Warp workspace
    Config(ConfigCommand),
    /// Build the current workspace
    Build(BuildCommand),
    /// Execute the 'Auto Deploy' script for the workspace (see Warp.toml)
    Deploy(AutoDeployCommand),
    /// Scaffold a new contract
    New(NewCommand),
    /// [WIP] Start the local validator node
    Node(NodeCommand),
    /// Run the JavaScript tests from the '/tests/' directory
    Test(TestCommand),
    /// Wasm commands for interacting with deployed contracts
    Wasm(WasmCommand),
}

fn main() -> Result<(), WarpError> {
    let cli = Cli::parse();

    let (project_root, config) = utils::project_config::ProjectConfig::parse_project_config()
        .map_or((None, None), |x| (Some(x.0), Some(x.1)));
    let profile: Box<dyn ChainProfile> = Box::new(ArchwayProfile);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    let result = match &cli.command {
        Commands::Deploy(x) => x.execute(project_root, config, &profile),
        Commands::Init(x) => x.execute(project_root, config, &profile),
        Commands::New(x) => x.execute(project_root, config, &profile),
        Commands::Build(x) => x.execute(project_root, config, &profile),
        Commands::Test(x) => x.execute(project_root, config, &profile),
        Commands::Node(x) => x.execute(project_root, config, &profile),
        Commands::Config(x) => x.execute(project_root, config, &profile),
        Commands::Wasm(x) => x.execute(project_root, config, &profile),
    };
    if let Err(x) = result {
        println!("{} {}", "Error!".red(), x.to_string().bright_red());
    }
    Ok(())
}
