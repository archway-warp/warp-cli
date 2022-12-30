use std::{
    collections::BTreeMap,
    io::Write,
    process::{Command, Output, Stdio},
    time::Duration,
};

use crate::{
    error::WarpError,
    executable::Executable,
    secretcli::{keys_show::KeysShowResponse, tx_query::TxQueryResponse},
    utils::project_config::{AutoDeployStep, ProjectConfig},
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
            Self::get_key_info(&config.autodeploy.account_id, password)?.address;

        println!("Deploying from: {}", &deployment_account);

        println!("Uploading contracts to the chain...");
        let mut store_txs: Vec<(AutoDeployStep, String)> = vec![];
        for step in config.autodeploy.steps.iter() {
            print!(" => {}", step.contract);
            let response =
                Self::store_contract(&step.contract, &config.autodeploy.account_id, password)?;
            let full_store_tx = Self::query_tx(&response.txhash)?;
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
                l.push_str(&rand::random::<u64>().to_string());
                l
            } else {
                step.label.clone()
            };

            let init_tx = Self::instantiate_contract(
                &code_id,
                &config.autodeploy.account_id,
                &label,
                &init_msg,
                step.coins,
                password,
            )?
            .txhash;
            let init_full_tx = Self::query_tx(&init_tx)?;
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

    fn get_common_cli_args<'a>(tx: bool) -> Vec<&'a str> {
        let mut args = vec!["--output", "json"];
        if tx {
            let mut tx_args = vec!["--gas", "1000000", "-y", "-b", "sync"];
            args.append(&mut tx_args);
        }
        args
    }

    fn call_process_with_input(cmd: &mut Command, input: &str) -> Result<Output, WarpError> {
        let child = cmd.spawn()?;

        {
            let mut stdin = child.stdin.as_ref().unwrap();
            stdin.write_all(input.as_bytes())?;
            stdin.flush()?;
        }
        let out = child.wait_with_output()?;
        Ok(out)
    }

    fn get_key_info(
        account_id: &str,
        password: Option<&str>,
    ) -> Result<KeysShowResponse, WarpError> {
        let mut tx = Command::new("secretcli");
        tx.args(vec!["keys", "show", account_id])
            .args(Self::get_common_cli_args(false))
            .stdout(Stdio::piped())
            .stdin(if password.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = Self::call_process_with_input(&mut tx, pass)?;
            json_data = out.stdout;
        } else {
            let out = tx.output()?;
            let bytes = out.stdout.clone();
            json_data = bytes;
        }
        let response: KeysShowResponse = serde_json::from_slice(json_data.as_slice())?;
        Ok(response)
    }

    fn store_contract(
        contract: &str,
        from: &str,
        password: Option<&str>,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("secretcli");
        tx.args(vec!["tx", "compute", "store", contract, "--from", from])
            .args(Self::get_common_cli_args(true))
            .stdout(Stdio::piped())
            .current_dir(ProjectConfig::find_project_root()?)
            .stdin(if password.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = Self::call_process_with_input(&mut tx, pass)?;
            json_data = out.stdout;
        } else {
            let out = tx.output()?;
            let bytes = out.stdout.clone();
            json_data = bytes;
        }
        let response: TxQueryResponse = serde_json::from_slice(json_data.as_slice())?;
        if response.code != 0 {
            return Err(WarpError::TxFailed(response.txhash, response.raw_log));
        }
        Ok(response)
    }

    fn instantiate_contract(
        code_id: &str,
        from: &str,
        label: &str,
        init_msg: &str,
        coins: Option<String>,
        password: Option<&str>,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("secretcli");
        tx.args(vec![
            "tx",
            "compute",
            "instantiate",
            code_id,
            init_msg,
            "--from",
            from,
            "--label",
            label,
            "--amount",
            &coins.unwrap_or_default(),
        ])
        .args(Self::get_common_cli_args(true))
        .stdout(Stdio::piped())
        .stdin(if password.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = Self::call_process_with_input(&mut tx, pass)?;
            json_data = out.stdout;
        } else {
            let out = tx.output()?;
            let bytes = out.stdout.clone();
            json_data = bytes;
        }
        let response: TxQueryResponse = serde_json::from_slice(json_data.as_slice())?;
        if response.code != 0 {
            return Err(WarpError::TxFailed(response.txhash, response.raw_log));
        }
        Ok(response)
    }

    // TODO: Make this ugly thing go away once a better solution is confirmed to be working
    fn query_tx(tx_hash: &str) -> Result<TxQueryResponse, WarpError> {
        let mut retries = 3;
        loop {
            let cmd = Command::new("secretcli")
                .args(vec!["q", "tx", tx_hash])
                .args(Self::get_common_cli_args(false))
                .stdin(Stdio::inherit())
                .output()?;
            let tx = cmd.stdout;
            if cmd.stderr.len() > 0 && retries > 0 {
                // crude but will do for beta
                retries -= 1;
                std::thread::sleep(Duration::from_millis(2000));
                continue;
            }
            let response: TxQueryResponse = serde_json::from_slice(tx.as_slice())?;
            if response.code != 0 {
                return Err(WarpError::TxFailed(response.txhash, response.raw_log));
            }
            return Ok(response);
        }
    }
}
