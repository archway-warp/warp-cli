use std::path::PathBuf;

use crate::{
    chains::chain_profile::ChainProfile,
    error::WarpError,
    executable::Executable,
    utils::project_config::{AutoDeployStep, ProjectConfig},
};
use clap::Args;
use regex::Regex;

#[derive(Args)]
pub struct NewCommand {
    //#[arg(short, long)]
    /// The name of the new contract
    pub name: String,
    /// Customize contract label
    #[arg(short, long)]
    pub label: Option<String>,
}

impl Executable for NewCommand {
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
        let mut config = config.unwrap().clone();

        let contract_name = Self::optimize_for_path(&self.name)?;
        let contract_dir = project_root.join("contracts").join(&contract_name);
        let deploy_step = AutoDeployStep {
            id: format!("$_{}", &self.name),
            contract: format!("artifacts/{}.wasm", &self.name),
            label: self.label.as_ref().unwrap_or(&self.name).to_string(),
            store_only: false,
            init_msg: "{ \"owner\": \"$account_id\", \"message\": \"\" }".to_owned(),
            migrate_msg: Some("{}".to_owned()),
            coins: None,
        };
        config.autodeploy.steps.push(deploy_step);
        profile.new_contract(&contract_name, &contract_dir, &project_root)?;
        config.save_project_config()?;

        Ok(())
    }
}

impl NewCommand {
    fn optimize_for_path(s: &str) -> Result<String, WarpError> {
        let rx = Regex::new(r"[^a-zA-Z0-9_]")?;
        let replaced = rx.replace_all(s, "_");
        Ok(replaced.trim_matches('_').to_lowercase().to_owned()) // we're not doing Python here
    }
}
mod tests {

    #[test]
    fn path_test() {
        let paths = vec![
            ("path-to-file", "path_to_file"),
            ("path to file 2", "path_to_file_2"),
            (r"path/to\file", "path_to_file"),
            ("Path[]tO][file()", "path__to__file"),
            ("path_to_file", "path_to_file"),
        ];
        for path in paths.iter() {
            let result = super::NewCommand::optimize_for_path(path.0).unwrap();
            assert_eq!(path.1, result);
        }
    }
}
