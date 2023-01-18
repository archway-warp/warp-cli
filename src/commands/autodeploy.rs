use std::{collections::BTreeMap, time::Duration};

use crate::{
    error::WarpError,
    executable::Executable,
    utils::{
        project_config::{AutoDeployStep, ProjectConfig},
        secretcli_util,
    },
};
use clap::Args;

#[derive(Args)]
pub struct AutoDeployCommand {
    //#[arg(short, long)]
    // The name of the new contract
    //pub name: String,
}

impl Executable for AutoDeployCommand {
    fn execute(&self) -> Result<(), WarpError> {
        let (_root, config) = ProjectConfig::parse_project_config()?;

        let password =
            rpassword::prompt_password("Enter your keyring password (if using/needed):")?;
        let password = if password.is_empty() {
            None
        } else {
            Some(password.as_str())
        };
        let deployment_account =
            secretcli_util::get_key_info(&config.autodeploy.account_id, password)?.address;

        println!("Deploying from: {}", &deployment_account);

        println!("Uploading contracts to the chain...");
        let mut store_txs: Vec<(AutoDeployStep, String)> = vec![];
        for step in config.autodeploy.steps.iter() {
            print!(" => {}", step.contract);
            let response = secretcli_util::store_contract(
                &step.contract,
                &config.autodeploy.account_id,
                password,
            )?;
            let full_store_tx = secretcli_util::query_tx(&response.txhash)?;
            let code_id = full_store_tx
                .logs
                .get(0)
                .unwrap()
                .events
                .first()
                .unwrap()
                .attributes
                .last()
                .unwrap()
                .value
                .clone();
            println!("\tDone ({}) - CODE: {}", &response.txhash, code_id);

            store_txs.push((step.clone(), code_id));
        }
        // Only add the extra wait if deploying one contract since otherwise it'll be fine anyway
        if store_txs.len() == 1 {
            std::thread::sleep(Duration::from_millis(4500));
        }
        println!("Instantiating uploaded contracts...");
        let mut contract_addresses: BTreeMap<String, String> = BTreeMap::new();
        for (step, code_id) in store_txs {
            print!(" => {}", &step.contract);
            let init_msg =
                Self::format_init_message(&step.init_msg, &contract_addresses, &deployment_account);
            let label = if config.autodeploy.make_labels_unique {
                let mut l = step.label.clone();
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
                step.label.clone()
            };

            let init_tx = secretcli_util::instantiate_contract(
                &code_id,
                &config.autodeploy.account_id,
                &label,
                &init_msg,
                step.coins,
                password,
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
            contract_addresses.insert(step.id, addr.clone());
            println!("\tDone ({}) - {}", &addr, &init_msg);
        }
        Ok(())
    }
}

impl AutoDeployCommand {
    fn format_init_message(
        init_msg: &str,
        addresses: &BTreeMap<String, String>,
        deployment_account: &str,
    ) -> String {
        let mut new_msg = init_msg.replace("$account_id", &deployment_account);
        addresses
            .iter()
            .for_each(|(k, v)| new_msg = new_msg.replace(k, v));
        new_msg
    }
}
