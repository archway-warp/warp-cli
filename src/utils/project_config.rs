use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::error::WarpError;

pub const CONFIG_FILENAME: &str = "Warp.toml";

#[derive(Serialize, Deserialize)]
pub struct ProjectConfig {
    pub tests: TestConfig,
    pub autodeploy: AutoDeploy,
}

#[derive(Serialize, Deserialize)]
pub struct TestConfig {
    pub node_setup_time: u16,
    pub test_container_name: String,
    pub persist_image: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AutoDeploy {
    pub account_id: String,
    pub make_labels_unique: bool,
    pub steps: Vec<AutoDeployStep>,
}

#[derive(Serialize, Deserialize)]
pub struct AutoDeployStep {
    pub id: String,
    pub contract: String,
    pub label: String,
    pub init_msg: String,
}

impl ProjectConfig {
    pub fn generate_and_save(path: PathBuf) -> Result<(), WarpError> {
        let toml_path = path.join(CONFIG_FILENAME);
        if toml_path.exists() {
            return Err(WarpError::ProjectFileAlreadyExists(toml_path));
        }
        let config = Self {
            tests: TestConfig {
                node_setup_time: 8,
                test_container_name: format!(
                    "secretdev-{}",
                    path.file_name().unwrap().to_str().unwrap()
                ),
                persist_image: false,
            },
            autodeploy: AutoDeploy {
                account_id: String::new(),
                make_labels_unique: true,
                steps: vec![],
            },
        };
        println!(
            "Project dir: {}",
            path.clone().as_os_str().to_str().unwrap()
        );
        fs::create_dir_all(path.clone())?;
        let toml = toml::to_string_pretty(&config)?;
        let mut file = File::create(toml_path)?;
        write!(&mut file, "{}", toml)?;
        Ok(())
    }

    pub fn parse_project_config() -> Result<(PathBuf, Self), WarpError> {
        let mut current_dir = std::env::current_dir()?;
        let mut config: ProjectConfig;
        loop {
            let project_file = current_dir.join(CONFIG_FILENAME);
            if project_file.exists() {
                config = toml::from_str(fs::read_to_string(project_file)?.as_str())?;
                return Ok((current_dir, config));
            }
            let parent = current_dir.parent();
            if let Some(parent) = parent {
                current_dir = parent.into();
            } else {
                return Err(WarpError::ProjectFileNotFound);
            };
        }
    }
}
