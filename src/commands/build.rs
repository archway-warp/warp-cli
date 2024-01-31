use std::process::Command;

pub use clap::{arg, Args};

use crate::{error::WarpError, executable::Executable, utils::project_config::ProjectConfig};

#[derive(Args)]
pub struct BuildCommand {
    /// Build for production with 'workspace-optimizer' docker image
    #[arg(default_value_t = false, short, long)]
    pub optimized: bool,
}

impl Executable for BuildCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let (root, config) = ProjectConfig::parse_project_config()?;
        if self.optimized {
            let rename_files: bool;
            let cmd_str = match config.tooling.optimizer_backend.as_str() {
                "cw-optimizoor" => {
                    rename_files = true;
                    format!("cargo cw-optimizoor .")
                }
                _ => {
                    rename_files = false;
                    format!("docker run --rm -v {0}:/code --mount type=volume,source={1}_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry cosmwasm/workspace-optimizer:0.12.10", 
                    &root.to_str().unwrap(),
                    &root.to_str().unwrap().rsplit("/").next().unwrap())
                }
            };
            let cmd_tokens = cmd_str.split(" ").collect::<Vec<&str>>();
            let cmd_name = cmd_tokens.get(0).unwrap();
            let cmd_args = cmd_tokens.iter().skip(1).map(|x| *x).collect::<Vec<&str>>();

            Command::new(cmd_name)
                .current_dir(&root)
                .args(cmd_args)
                .spawn()?
                .wait()?;
            if rename_files {
                let artifacts = root.clone().join("artifacts");
                let dir = std::fs::read_dir(&artifacts)?;
                // TODO: filter_map()
                let files_to_rename = dir
                    .filter(|x| x.is_ok())
                    .map(|x| x.unwrap())
                    .map(|x| x.file_name())
                    .filter(|x| x.to_str().unwrap().contains("-x86_64"))
                    .collect::<Vec<_>>();
                for file in files_to_rename {
                    let new_name = artifacts
                        .clone()
                        .join(file.to_str().unwrap().replace("-x86_64", ""));
                    std::fs::rename(artifacts.clone().join(&file), new_name)?;
                }
            }
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
