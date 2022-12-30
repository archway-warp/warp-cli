use std::{env, process::Command};

pub use clap::{arg, Args};

use crate::{error::WarpError, executable::Executable};

#[derive(Args)]
pub struct BuildCommand {
    /// Build for production with 'workspace-optimizer' docker image
    #[arg(default_value_t = false, short, long)]
    pub optimized: bool,
}

impl Executable for BuildCommand {
    fn execute(&self) -> Result<(), WarpError> {
        if self.optimized {
            let cmd_str = format!("docker run --rm -v {0}:/code --mount type=volume,source={1}_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.10", 
                env::current_dir()?.to_str().unwrap(),
                env::current_dir()?.to_str().unwrap().rsplit("/").next().unwrap());
            let cmd_tokens = cmd_str.split(" ").collect::<Vec<&str>>();
            let cmd_name = cmd_tokens.get(0).unwrap();
            let cmd_args = cmd_tokens.iter().skip(1).map(|x| *x).collect::<Vec<&str>>();

            Command::new(cmd_name).args(cmd_args).spawn()?.wait()?;
        } else {
            let cmd = Command::new("cargo")
                .arg("build")
                .arg("--target")
                .arg("wasm32-unknown-unknown")
                .env("RUSTFLAGS", "-C link-arg=-s")
                .spawn();
            cmd?.wait()?;
        }
        Ok(())
    }
}
