mod archway;
mod commands;
mod consts;
mod error;
mod executable;
mod utils;

use clap::{command, Parser, Subcommand};
use commands::{
    autodeploy::AutoDeployCommand, build::BuildCommand, config::ConfigCommand, init::InitCommand,
    new::NewCommand, node::NodeCommand, test::TestCommand, wasm::WasmCommand,
};
use error::WarpError;
use executable::Executable;

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

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Deploy(x) => x.execute(),
        Commands::Init(x) => x.execute(),
        Commands::New(x) => x.execute(),
        Commands::Build(x) => x.execute(),
        Commands::Test(x) => x.execute(),
        Commands::Node(x) => x.execute(),
        Commands::Config(x) => x.execute(),
        Commands::Wasm(x) => x.execute(),
    }?;
    Ok(())
}
