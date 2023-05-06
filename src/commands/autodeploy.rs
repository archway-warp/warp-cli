use std::{collections::BTreeMap, time::Duration};

use crate::{
    error::WarpError,
    executable::Executable,
    utils::{
        deployment_task::DeploymentTask,
        project_config::{AutoDeployStep, ProjectConfig},
        secretcli_util,
    },
};
use clap::Args;
use owo_colors::OwoColorize;

#[derive(Args)]
pub struct AutoDeployCommand {
    //#[arg(short, long)]
    // The name of the new contract
    //pub name: String,
}

impl Executable for AutoDeployCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let (_root, config) = ProjectConfig::parse_project_config()?;
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
        let deployment_account =
            secretcli_util::get_key_info(&config.autodeploy.account_id, password, &config)?.address;

        println!("Deploying from: {}", &deployment_account);

        println!("Uploading contracts to the chain...");
        let mut store_txs: Vec<DeploymentTask> = vec![];
        for step in config.autodeploy.steps.iter() {
            print!(" {} {}", "=>".bright_yellow(), step.contract.bright_blue());
            let response = secretcli_util::store_contract(
                &step.contract,
                &config.autodeploy.account_id,
                password,
                &config,
            )?;
            let full_store_tx = secretcli_util::query_tx(&response.txhash)?;
            let code_id = full_store_tx
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
            let init_tx = secretcli_util::instantiate_contract(
                t.code_id.as_ref().unwrap(),
                &config.autodeploy.account_id,
                &deployment_account,
                &label,
                &init_msg,
                task.coins.clone(),
                password,
                &config,
            )?
            .txhash;
            let init_full_tx = secretcli_util::query_tx(&init_tx)?;
            let addr = init_full_tx
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
            println!(
                "\t{} ({}) -- '{}'",
                "Done.".bright_green(),
                &addr.bright_cyan(),
                &init_msg.bright_yellow()
            );
        }
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
