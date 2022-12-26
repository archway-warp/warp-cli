use std::{fs::File, io::Write, path::PathBuf, process::Command};

use crate::{
    error::WarpError,
    executable::Executable,
    utils::{self, project_config::ProjectConfig},
};
use clap::Args;

#[derive(Args)]
pub struct AutoDeployCommand {
    //#[arg(short, long)]
    // The name of the new contract
    //pub name: String,
}

impl Executable for AutoDeployCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let (root, config) = ProjectConfig::parse_project_config()?;
        

        Ok(())
    }
}
