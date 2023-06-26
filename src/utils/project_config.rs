use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use crate::error::WarpError;

pub const CONFIG_FILENAME: &str = "Warp.toml";

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub network: Network,
    pub tooling: Tooling,
    pub tests: TestConfig,
    pub autodeploy: AutoDeploy,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Network {
    pub chain_id: String,
    pub rpc_url: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tooling {
    /// Backend for optimizing contracts for production
    pub optimizer_backend: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TestConfig {
    pub node_setup_time: u16,
    pub test_container_name: String,
    pub persist_image: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutoDeploy {
    pub account_id: String,
    pub make_labels_unique: bool,
    pub steps: Vec<AutoDeployStep>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AutoDeployStep {
    pub id: String,
    pub contract: String,
    pub label: String,
    pub store_only: bool,
    pub migrate_msg: Option<String>,
    pub init_msg: String,
    pub coins: Option<String>,
}

impl ProjectConfig {
    pub fn empty() -> ProjectConfig {
        ProjectConfig {
            network: Network {
                chain_id: String::new(),
                rpc_url: String::new(),
                denom: String::new(),
            },
            tooling: Tooling {
                optimizer_backend: String::new(),
            },
            tests: TestConfig {
                node_setup_time: 0,
                test_container_name: String::new(),
                persist_image: false,
            },
            autodeploy: AutoDeploy {
                account_id: String::new(),
                make_labels_unique: false,
                steps: vec![],
            },
        }
    }
    pub fn generate_and_save(path: PathBuf) -> Result<(), WarpError> {
        let toml_path = path.join(CONFIG_FILENAME);
        if toml_path.exists() {
            return Err(WarpError::ProjectFileAlreadyExists(toml_path));
        }
        let config = Self {
            network: Network {
                rpc_url: "https://rpc.constantine.archway.tech:443".to_string(),
                denom: "aconst".to_string(),
                chain_id: "constantine-3".to_string(),
            },
            tooling: Tooling {
                optimizer_backend: "default".to_owned(),
            },
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

    pub fn find_project_root() -> Result<PathBuf, WarpError> {
        let mut current_dir = std::env::current_dir()?;
        loop {
            let project_file = current_dir.join(CONFIG_FILENAME);
            if project_file.exists() {
                return Ok(current_dir);
            }
            let parent = current_dir.parent();
            if let Some(parent) = parent {
                current_dir = parent.into();
            } else {
                return Err(WarpError::ProjectFileNotFound);
            };
        }
    }

    pub fn parse_project_config() -> Result<(PathBuf, Self), WarpError> {
        let mut current_dir = std::env::current_dir()?;
        let config: ProjectConfig;
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

    pub fn save_project_config(&self) -> Result<(), WarpError> {
        let toml_path = Self::find_project_root()?.join(CONFIG_FILENAME);
        std::fs::write(toml_path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
