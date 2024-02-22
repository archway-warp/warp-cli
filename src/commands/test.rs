use std::{path::PathBuf, process::Command, time::Duration};

use clap::Args;

use crate::{
    chains::chain_profile::ChainProfile, error::WarpError, executable::Executable,
    utils::project_config::ProjectConfig,
};

use super::{node::NodeCommand, BuildCommand};

#[derive(Args)]
pub struct TestCommand {
    /// Rebuild the contracts before running tests
    #[arg(default_value_t = false, short, long)]
    pub rebuild: bool,
    /// Don't start a new instance of localsecret for this testing session
    #[arg(default_value_t = false, short, long)]
    pub skip_environment: bool,
}

impl Executable for TestCommand {
    fn execute(
        &self,
        project_root: Option<PathBuf>,
        config: Option<ProjectConfig>,
        profile: &Box<dyn ChainProfile>,
    ) -> Result<(), WarpError> {
        if project_root.is_none() {
            return Err(WarpError::ProjectFileNotFound);
        };
        let project_root = project_root.unwrap();
        let config = config.unwrap();

        // 1. Build the code if requested
        if self.rebuild {
            let cmd = BuildCommand { optimized: true };
            cmd.execute(Some(project_root.clone()), Some(config.clone()), profile)?;
        }

        // 2. Set up the node unless specified otherwise
        let node_cleanup: bool;
        if !self.skip_environment {
            let cmd = NodeCommand {
                detached: true,
                container: Some(config.tests.test_container_name.clone()),
                persistant: config.tests.persist_image,
            };
            let status = cmd.execute(Some(project_root.clone()), Some(config.clone()), profile);
            let is_conflict = match status {
                Ok(_) => false,
                Err(x) => {
                    println!("{:?}", x);
                    if let WarpError::NodeStartupError(_x) = x {
                        // TODO: For now we're assuming we got conflict. We should explicitly check in the future.
                        println!("Possible container conflict.");
                        true
                    } else {
                        false
                    }
                }
            };
            if is_conflict {
                // If the container exists, just start it back up with the old state kept
                let mut cmd = Command::new("docker")
                    .arg("container")
                    .arg("start")
                    .arg(config.tests.test_container_name.clone())
                    .spawn()?;
                cmd.wait()?;
            }
            println!("Waiting for the node to start producing blocks...");
            std::thread::sleep(Duration::from_secs(config.tests.node_setup_time as u64));
            node_cleanup = true;
        } else {
            node_cleanup = false;
        }
        let mut test = Command::new("yarn")
            .arg("run")
            .arg("ts-mocha")
            .arg("-p")
            .arg(project_root.clone().join("tests/tsconfig.json"))
            .arg("-t")
            .arg("100000")
            .arg(
                project_root
                    .clone()
                    .join("tests")
                    .join("src")
                    .join("**")
                    .join("*.test.ts"),
            )
            .current_dir(project_root.clone().join("tests"))
            .spawn()?;
        test.wait()?;

        if node_cleanup {
            self.node_cleanup(&config)?;
        }
        Ok(())
    }
}

impl TestCommand {
    fn node_cleanup(&self, config: &ProjectConfig) -> Result<(), WarpError> {
        let mut process = Command::new("docker")
            .arg("container")
            .arg("stop")
            .arg(config.tests.test_container_name.clone())
            .spawn()?;
        process.wait()?;

        if config.tests.persist_image == false {
            let logs = Command::new("docker")
                .args(vec![
                    "container",
                    "logs",
                    "--tail",
                    "1",
                    &config.tests.test_container_name,
                ])
                .output()?;
            if logs.status.success() {
                let mut rm = Command::new("docker")
                    .args(vec!["container", "rm", &config.tests.test_container_name])
                    .spawn()?;
                rm.wait()?;
            }
        }
        Ok(())
    }
}
