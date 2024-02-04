use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use serde_json::Value;

use crate::{
    archway::{
        estimate_fees::EstimateFeesResponse, keys_show::KeysShowResponse, tx_query::TxQueryResponse,
    },
    error::WarpError,
};

use crate::utils::{command_util::CommandWithInput, project_config::ProjectConfig};

use super::chain_profile::ChainProfile;

pub struct ArchwayProfile;

impl ArchwayProfile {
    fn get_estimated_fee(&self, config: &ProjectConfig) -> Result<EstimateFeesResponse, WarpError> {
        let mut cmd = Command::new("archwayd");
        cmd.args(vec!["q", "rewards", "estimate-fees", "1"])
            .args(self.get_common_cli_args(false, true, false, config))
            .stdin(Stdio::inherit());
        let output = cmd.output()?;

        let tx = output.stdout;
        let response = serde_json::from_slice::<EstimateFeesResponse>(&tx)?;
        Ok(response)
    }
}

impl ChainProfile for ArchwayProfile {
    fn get_common_cli_args<'a, 'b>(
        &self,
        tx: bool,
        network: bool,
        store: bool,
        config: &'b ProjectConfig,
    ) -> Vec<String> {
        let mut args = vec!["--output".to_string(), "json".to_string()];
        if network {
            args.push("--chain-id".to_string());
            args.push(config.network.chain_id.to_string());
            args.push("--node".to_string());
            args.push(config.network.rpc_url.to_string());
        }
        if tx {
            let fee: EstimateFeesResponse = self.get_estimated_fee(config).unwrap();
            let mut tx_args = vec![
                "-y".to_string(),
                "-b".to_string(),
                "block".to_string(),
                "--gas".to_string(),
                "auto".to_string(),
                "--gas-adjustment".to_string(),
                if store {
                    "2".to_string()
                } else {
                    "1.4".to_string()
                },
                "--gas-prices".to_string(),
                fee.get_gas_price(),
            ];
            args.append(&mut tx_args);
        }
        args
    }

    fn get_key_info(
        &self,
        account_id: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<KeysShowResponse, WarpError> {
        let mut tx = Command::new("archwayd");
        tx.args(vec!["keys", "show", account_id])
            .args(self.get_common_cli_args(false, false, false, config))
            .stdout(Stdio::piped())
            .stdin(if password.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = tx.call_process_with_input(pass)?;
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
        &self,
        contract: &str,
        from: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("archwayd");
        tx.args(vec!["tx", "wasm", "store", contract, "--from", from])
            .args(self.get_common_cli_args(true, true, true, config))
            .stdout(Stdio::piped())
            .current_dir(ProjectConfig::find_project_root()?)
            .stdin(if password.is_some() {
                Stdio::piped()
            } else {
                Stdio::inherit()
            });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = tx.call_process_with_input(pass)?;
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
        &self,
        code_id: &str,
        from: &str,
        admin: &str,
        label: &str,
        init_msg: &str,
        coins: Option<String>,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("archwayd");
        tx.args(vec![
            "tx",
            "wasm",
            "instantiate",
            code_id,
            init_msg,
            "--from",
            from,
            "--label",
            label,
            "--amount",
            &coins.unwrap_or_default(),
            "--admin",
            admin,
        ])
        .args(self.get_common_cli_args(true, true, false, config))
        .stdout(Stdio::piped())
        .stdin(if password.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = tx.call_process_with_input(pass)?;
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

    fn execute_contract(
        &self,
        contract_address: &str,
        msg: &str,
        from: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("archwayd");
        tx.args(vec![
            "tx",
            "wasm",
            "execute",
            contract_address,
            msg,
            "--from",
            from,
        ])
        .args(self.get_common_cli_args(true, true, false, config))
        .stdout(Stdio::piped())
        .stdin(if password.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = tx.call_process_with_input(pass)?;
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

    fn migrate_contract(
        &self,
        contract_address: &str,
        code_id: &str,
        from: &str,
        migrate_msg: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut tx = Command::new("archwayd");
        tx.args(vec![
            "tx",
            "wasm",
            "migrate",
            contract_address,
            code_id,
            migrate_msg,
            "--from",
            from,
        ])
        .args(self.get_common_cli_args(true, true, false, config))
        .stdout(Stdio::piped())
        .stdin(if password.is_some() {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        let json_data: Vec<u8>;
        if let Some(pass) = password {
            let out = tx.call_process_with_input(pass)?;
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
    fn query_tx(
        &self,
        tx_hash: &str,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError> {
        let mut retries = 10;
        loop {
            let cmd = Command::new("archwayd")
                .args(vec!["q", "tx", tx_hash])
                .args(self.get_common_cli_args(false, true, false, config))
                .stdin(Stdio::inherit())
                .output()?;
            let tx = cmd.stdout;
            if cmd.stderr.len() > 0 && retries > 0 {
                // crude but will do for beta
                retries -= 1;
                std::thread::sleep(Duration::from_millis(1600));
                continue;
            }
            println!("TX(L{}): {}", tx.len(), String::from_utf8(tx.clone())?);
            let response: TxQueryResponse = serde_json::from_slice(tx.as_slice())?;
            if response.code != 0 {
                return Err(WarpError::TxFailed(response.txhash, response.raw_log));
            }
            return Ok(response);
        }
    }

    fn query_contract_smart(
        &self,
        contract: &str,
        query: &str,
        config: &ProjectConfig,
    ) -> Result<Value, WarpError> {
        let cmd = Command::new("archwayd")
            .args(vec![
                "q",
                "wasm",
                "contract-state",
                "smart",
                contract,
                query,
            ])
            .args(self.get_common_cli_args(false, true, false, config))
            .stdin(Stdio::inherit())
            .output()?;
        let tx = cmd.stdout;
        if cmd.stderr.len() > 0 {
            let msg = String::from_utf8(cmd.stderr)?;
            return Err(WarpError::UnderlyingCliError(msg));
        }
        let response: Value = serde_json::from_slice(tx.as_slice())?;
        return Ok(response);
    }

    fn init_project(&self, dir: &PathBuf) -> Result<(), WarpError> {
        println!("Initializing new workspace...");
        let cmd = Command::new("git")
            .arg("clone")
            .arg("https://github.com/archway-warp/warp-template.git")
            .arg(dir.as_os_str())
            .stdout(Stdio::null())
            .spawn()?
            .wait()?;
        if cmd.success() {
            return Ok(());
        } else {
            return Err(WarpError::InitFailed);
        }
    }
}
