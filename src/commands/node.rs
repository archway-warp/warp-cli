use std::{path::PathBuf, process::Command};

use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, executable::Executable,
    utils::project_config::ProjectConfig,
};
use clap::Args;

#[derive(Args)]
pub struct NodeCommand {
    #[arg(default_value_t = false, short, long)]
    pub detached: bool,
    #[arg(short, long)]
    pub container: Option<String>,
    #[arg(default_value_t = false, short, long)]
    pub persistant: bool,
}

impl Executable for NodeCommand {
    fn execute(
        &self,
        project_root: Option<PathBuf>,
        config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        if project_root.is_none() {
            return Err(WarpError::ProjectFileNotFound);
        };
        let config = config.unwrap().clone();

        let cmd_str = profile.get_node_docker_command(self.container.clone(), &config);
        let cmd_tokens = cmd_str.split(" ").collect::<Vec<&str>>();
        let cmd_name = *cmd_tokens.get(0).unwrap();
        let mut cmd_args = cmd_tokens.iter().skip(1).map(|x| *x).collect::<Vec<&str>>();

        let mut cmd = Command::new(cmd_name);
        if !self.persistant {
            cmd_args.insert(2, "--rm");
        }
        if self.detached {
            cmd_args.insert(1, "-d");
        }
        cmd.args(cmd_args);
        let mut node = cmd.spawn()?;
        let status_code = node.wait()?;
        if status_code.success() {
            Ok(())
        } else {
            Err(WarpError::NodeStartupError(status_code))
        }
    }
}
