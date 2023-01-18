use std::{fs::File, io::Write, path::Path};

use crate::{
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
    fn execute(&self) -> Result<(), WarpError> {
        let (root, mut config) = ProjectConfig::parse_project_config()?;
        let contract_name = Self::optimize_for_path(&self.name)?;
        println!("[1/3] Downloading contract files...");
        let contract_dir = root.join("contracts").join(&contract_name);
        std::fs::create_dir_all(contract_dir.clone())?;
        let clone = std::process::Command::new("git")
            .args(vec![
                "clone",
                "--depth=1",
                "https://github.com/secret-warp/contract-template.git",
                contract_dir.clone().as_os_str().to_str().unwrap(),
                "-q",
            ])
            .spawn()?
            .wait()?;
        if !clone.success() {
            return Err(WarpError::ContractTemplateCloneFailed);
        }

        std::fs::remove_dir_all(contract_dir.clone().join(".git"))?;
        std::fs::remove_file(contract_dir.clone().join("README.md"))?;
        let cargo_path = contract_dir.clone().join("Cargo.toml");
        Self::replace_in_file(cargo_path, "<CONTRACT_NAME>", &contract_name)?;

        let lib_path = contract_dir.clone().join("src").join("contract.rs");
        Self::replace_in_file(lib_path, "<CONTRACT_NAME>", &contract_name)?;

        let schema_path = contract_dir.clone().join("examples").join("schema.rs");
        Self::replace_in_file(schema_path, "<CONTRACT_NAME>", &contract_name)?;

        let shared_path = root.clone().join("packages").join("shared");
        let msg_path = shared_path
            .clone()
            .join("src")
            .join(&contract_name)
            .join("msg.rs");
        std::fs::create_dir_all(msg_path.clone().parent().unwrap())?;
        let mod_path = msg_path.clone().parent().unwrap().join("mod.rs");
        std::fs::write(msg_path, crate::consts::MSG_FILE)?;
        std::fs::write(mod_path, "pub mod msg;")?;
        let lib_path = shared_path.clone().join("src").join("lib.rs");
        let mut lib_file = File::options().write(true).append(true).open(lib_path)?;
        writeln!(&mut lib_file, "pub mod {};", &contract_name)?;

        println!("[2/3] Updating the deployment script...");
        let deploy_step = AutoDeployStep {
            id: format!("$_{}", &self.name),
            contract: format!("artifacts/{}.wasm", &self.name),
            label: self.label.as_ref().unwrap_or(&self.name).to_string(),
            init_msg: "{ \"owner\": \"$account_id\", \"message\": \"\" }".to_owned(),
            coins: None,
        };
        config.autodeploy.steps.push(deploy_step);
        config.save_project_config()?;
        println!("[2/2] Building the workspace...");
        std::process::Command::new("cargo")
            .arg("build")
            .current_dir(root)
            .spawn()?
            .wait()?;

        Ok(())
    }
}

impl NewCommand {
    fn optimize_for_path(s: &str) -> Result<String, WarpError> {
        let rx = Regex::new(r"[^a-zA-Z0-9_]")?;
        let replaced = rx.replace_all(s, "_");
        Ok(replaced.trim_matches('_').to_lowercase().to_owned()) // we're not doing Python here
    }

    fn replace_in_file<P>(path: P, find: &str, replace: &str) -> Result<(), WarpError>
    where
        P: AsRef<Path>,
    {
        let mut content = std::fs::read_to_string(&path)?;
        content = content.replace(&find, &replace);
        std::fs::write(&path, content)?;

        Ok(())
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
