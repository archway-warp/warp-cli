use std::{path::PathBuf, time::Duration};

use crate::{
    chains::chain_profile::ChainProfile,
    commands::BuildCommand,
    error::WarpError,
    executable::Executable,
    utils::{
        deployment_result::DeploymentResult, deployment_task::DeploymentTask,
        project_config::ProjectConfig,
    },
};
use clap::Args;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct AutoDeployCommand {
    #[arg(short, long)]
    /// The name of the new contract
    pub rebuild: bool,
}

impl Executable for AutoDeployCommand {
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

        if config.autodeploy.account_id.is_empty() {
            println!(
                "{} {}",
                "Warning!".bright_yellow(),
                "You did not specify a deployment account in Warp.toml (autodeploy.account_id)"
                    .yellow()
            );
            return Ok(());
        }

        let password =
            rpassword::prompt_password("Enter your keyring password (if using/needed):")?;
        let password = if password.is_empty() {
            None
        } else {
            Some(password.as_str())
        };

        if self.rebuild {
            BuildCommand { optimized: true }.execute(
                Some(project_root),
                Some(config.clone()),
                profile,
            )?;
        }

        let deployment_account = profile
            .get_key_info(&config.autodeploy.account_id, password, &config)?
            .address;

        println!("Deploying from: {}", &deployment_account);

        println!("Uploading contracts to the chain...");
        let mut store_txs: Vec<DeploymentTask> = vec![];
        for step in config.autodeploy.steps.iter() {
            print!(" {} {}", "=>".bright_yellow(), step.contract.bright_blue());
            let response = profile.store_contract(
                &step.contract,
                &config.autodeploy.account_id,
                password,
                &config,
            )?;
            // let full_tx = profile.query_tx(&response.txhash)?;
            let code_id = response
                .logs
                .last()
                .unwrap()
                .events
                .last()
                .unwrap()
                .attributes
                .last()
                .unwrap()
                .value
                .clone();
            println!(
                "\t{} ({}) - CODE: {}",
                "Done.".bright_green(),
                &response.txhash.bright_blue(),
                code_id.bright_green()
            );

            store_txs.push(DeploymentTask {
                step: &step,
                code_id: Some(code_id.clone()),
                contract_address: None,
            });
        }
        // Only add the extra wait if deploying one contract since otherwise it'll be fine anyway
        if store_txs.len() == 1 {
            std::thread::sleep(Duration::from_millis(4500));
        }
        println!("Instantiating uploaded contracts...");

        let deploy_existed = DeploymentResult::exists()?;
        let mut deployment_file = if deploy_existed {
            DeploymentResult::parse()?.1
        } else {
            DeploymentResult::default()
        };
        let current_network = deployment_file.network(&config.network.chain_id);

        for task in config.autodeploy.steps.iter() {
            if task.store_only {
                println!(
                    " {} {} {}",
                    "(X)".bright_yellow(),
                    &task.contract.bright_blue(),
                    "skipped.".bright_yellow()
                );
                continue;
            }
            let contract_addr: String;
            if !current_network.contains_key(&task.id) {
                print!(" {} {}", "=>".bright_yellow(), &task.contract.bright_blue());
                let init_msg =
                    Self::format_init_message(&task.init_msg, &store_txs, &deployment_account);
                let label = if config.autodeploy.make_labels_unique {
                    let mut l = task.label.clone();
                    l.push('-');
                    l.push_str(
                        &std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            .to_string(),
                    );
                    l
                } else {
                    task.label.clone()
                };
                let t = store_txs.iter_mut().find(|x| &x.step.id == &task.id);
                if t.is_none() {
                    break;
                }
                let t = t.unwrap();
                let init_tx = profile.instantiate_contract(
                    t.code_id.as_ref().unwrap(),
                    &config.autodeploy.account_id,
                    &deployment_account,
                    &label,
                    &init_msg,
                    task.coins.clone(),
                    password,
                    &config,
                )?;
                // let init_full_tx = profile.query_tx(&init_tx)?;
                let addr = init_tx
                    .logs
                    .first()
                    .unwrap()
                    .events
                    .first()
                    .unwrap()
                    .attributes
                    .iter()
                    .filter(|x| x.key.contains("address"))
                    .collect::<Vec<_>>()
                    .get(0)
                    .unwrap()
                    .value
                    .clone();
                t.contract_address = Some(addr.clone());
                contract_addr = addr.clone();
                println!(
                    "\t{} ({}) -- '{}'",
                    "Done.".bright_green(),
                    &addr.bright_cyan(),
                    &init_msg.bright_yellow()
                );
            } else {
                print!(" {} {}", "=>".bright_yellow(), &task.contract.bright_blue());

                let t = store_txs.iter_mut().find(|x| &x.step.id == &task.id);
                if t.is_none() {
                    break;
                }
                let t = t.unwrap();
                contract_addr = current_network.get(&task.id).unwrap().clone();
                t.contract_address = Some(contract_addr.clone());
                let _tx = profile.migrate_contract(
                    &contract_addr,
                    &t.code_id.as_ref().unwrap(),
                    &config.autodeploy.account_id,
                    &task.migrate_msg.as_ref().unwrap_or(&String::from("{}")),
                    password,
                    &config,
                )?;
                // let _full_tx = profile.query_tx(&tx.txhash)?;
                println!(
                    "\t{} (CODE ID: {} => {}) -- '{}'",
                    "Done.".bright_green(),
                    &t.code_id.as_ref().unwrap().bright_cyan(),
                    &contract_addr.bright_cyan(),
                    &task
                        .migrate_msg
                        .as_ref()
                        .unwrap_or(&String::from("{}"))
                        .bright_yellow()
                );
            }
            current_network
                .entry(task.id.clone())
                .or_insert(contract_addr);
        }
        deployment_file.save()?;
        Ok(())
    }
}

impl AutoDeployCommand {
    fn format_init_message(
        init_msg: &str,
        tasks: &[DeploymentTask],
        deployment_account: &str,
    ) -> String {
        let mut new_msg = init_msg.replace("$account_id", &deployment_account);
        tasks.iter().for_each(|x| {
            new_msg = new_msg
                .replace(
                    &format!("${}", &x.step.id),
                    &x.contract_address.as_ref().unwrap_or(&String::new()),
                )
                .replace(&format!("#{}", &x.step.id), &x.code_id.as_ref().unwrap())
        });
        new_msg
    }
}
