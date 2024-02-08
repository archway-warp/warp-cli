use std::{
    fs::File, io::Write, path::PathBuf, process::{Command, Stdio}, time::Duration
};

use serde_json::Value;

use crate::{
    archway::{
        keys_show::KeysShowResponse, tx_query::TxQueryResponse,
    }, commands::config::NetworkConfig, error::WarpError, utils::{file_util, project_config::Network}
};

use crate::utils::{command_util::CommandWithInput, project_config::ProjectConfig};

use super::chain_profile::ChainProfile;

pub struct XionProfile;

impl XionProfile {
}

impl ChainProfile for XionProfile {
    fn get_profile_name(&self) -> String {
        "xion".to_owned()
    }

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
                config.network.gas_prices.clone().unwrap_or("0.00025uxion".to_owned()),
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
        let mut tx = Command::new("xiond");
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
        let mut tx = Command::new("xiond");
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
        let mut tx = Command::new("xiond");
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
        let mut tx = Command::new("xiond");
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
        let mut tx = Command::new("xiond");
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
            let cmd = Command::new("xiond")
                .args(vec!["q", "tx", tx_hash])
                .args(self.get_common_cli_args(false, true, false, config))
                .stdin(Stdio::inherit())
                .output()?;
            let tx = cmd.stdout;
            if !cmd.stderr.is_empty() && retries > 0 {
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
        let cmd = Command::new("xiond")
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
        Ok(response)
    }

    fn init_project(&self, dir: &PathBuf) -> Result<(), WarpError> {
        println!("Initializing new workspace...");
        let cmd = Command::new("git")
            .arg("clone")
            .arg("https://github.com/xion-warp/warp-template.git")
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

    fn new_contract(
        &self,
        contract_name: &str,
        contract_dir: &PathBuf,
        project_root: &PathBuf,
    ) -> Result<(), WarpError> {
        println!("[1/2] Downloading contract files...");
        std::fs::create_dir_all(contract_dir.clone())?;
        let clone = std::process::Command::new("git")
            .args(vec![
                "clone",
                "--depth=1",
                "https://github.com/xion-warp/contract-template.git",
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
        file_util::replace_in_file(cargo_path, "<CONTRACT_NAME>", &contract_name)?;

        let lib_path = contract_dir.clone().join("src").join("contract.rs");
        file_util::replace_in_file(lib_path, "<CONTRACT_NAME>", &contract_name)?;

        let schema_path = contract_dir.clone().join("src").join("bin").join("schema.rs");
        file_util::replace_in_file(schema_path, "<CONTRACT_NAME>", &contract_name)?;

        let shared_path = project_root.clone().join("packages").join("shared");
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
        println!("[2/2] Building the workspace...");
        std::process::Command::new("cargo")
            .arg("build")
            .current_dir(project_root)
            .spawn()?
            .wait()?;
        Ok(())
    }

    fn get_node_docker_command(&self, container: Option<String>, config: &ProjectConfig) -> String {
        format!("docker run -it -p 9091:9091 -p 26657:26657 -p 26656:26656 -p 1317:1317 -p 5000:5000 -v {0}:/root/code --name {1} ghcr.io/scrtlabs/localsecret:v1.5.1",
            std::env::current_dir().unwrap().to_str().unwrap(), 
            container.clone().unwrap_or_else(|| config.tests.test_container_name.clone())
        )
    }

    fn network_params(&self, network_config: &NetworkConfig) -> Network {
        match network_config {
            NetworkConfig::Mainnet => todo!("Mainnet not yet supported."),
            NetworkConfig::Testnet => Network {
                profile: self.get_profile_name(),
                chain_id: "xion-local-testnet-1".to_owned(),
                rpc_url: "https://rpc.xion-testnet-1.burnt.com:443".to_owned(),
                denom: "uxion".to_owned(),
                gas_prices: Some("0uxion".to_owned()),
            },
            NetworkConfig::Local => Network {
                profile: self.get_profile_name(),
                chain_id: "xion-local-testnet-1".to_owned(),
                rpc_url: "".to_owned(), // TODO: Add local node URL
                denom: "uxion".to_owned(),
                gas_prices: Some("0.00025uxion".to_owned()),
            },
        }
    }
}
