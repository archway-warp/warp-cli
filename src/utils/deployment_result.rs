use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::WarpError;

use super::project_config::ProjectConfig;

pub const CONFIG_FILENAME: &str = "Deployment.toml";

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DeploymentResult {
    pub deployment: HashMap<String, HashMap<String, String>>,
}

impl DeploymentResult {
    pub fn exists() -> Result<bool, WarpError> {
        let path = ProjectConfig::find_project_root()?;
        Ok(path.join(CONFIG_FILENAME).exists())
    }

    pub fn parse() -> Result<(PathBuf, Self), WarpError> {
        let mut current_dir = std::env::current_dir()?;
        let config: Self;
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

    pub fn network(&mut self, id: &str) -> &mut HashMap<String, String> {
        self.deployment
            .entry(id.to_string())
            .or_insert(HashMap::new())
    }

    pub fn save(&self) -> Result<(), WarpError> {
        let toml_path = ProjectConfig::find_project_root()?.join(CONFIG_FILENAME);
        std::fs::write(toml_path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
