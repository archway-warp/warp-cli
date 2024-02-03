use std::{
    process::{Command, Stdio},
    time::Duration,
};

use serde_json::Value;

use crate::{
    archway::{
        estimate_fees::{EstimateFeesResponse, EstimatedFee},
        keys_show::KeysShowResponse,
        tx_query::TxQueryResponse,
    },
    error::WarpError,
};

use super::{command_util::CommandWithInput, project_config::ProjectConfig};

fn get_common_cli_args<'a, 'b>(
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
        let fee: EstimateFeesResponse = get_estimated_fee(config).unwrap();
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
pub fn get_estimated_fee(config: &ProjectConfig) -> Result<EstimateFeesResponse, WarpError> {
    let mut cmd = Command::new("archwayd");
    cmd.args(vec!["q", "rewards", "estimate-fees", "1"])
        .args(get_common_cli_args(false, true, false, config))
        .stdin(Stdio::inherit());
    let output = cmd.output()?;

    let tx = output.stdout;
    let response = serde_json::from_slice::<EstimateFeesResponse>(&tx)?;
    Ok(response)
}

pub fn get_key_info(
    account_id: &str,
    password: Option<&str>,
    config: &ProjectConfig,
) -> Result<KeysShowResponse, WarpError> {
    let mut tx = Command::new("archwayd");
    tx.args(vec!["keys", "show", account_id])
        .args(get_common_cli_args(false, false, false, config))
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

pub fn store_contract(
    contract: &str,
    from: &str,
    password: Option<&str>,
    config: &ProjectConfig,
) -> Result<TxQueryResponse, WarpError> {
    let mut tx = Command::new("archwayd");
    tx.args(vec!["tx", "wasm", "store", contract, "--from", from])
        .args(get_common_cli_args(true, true, true, config))
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

pub fn instantiate_contract(
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
    .args(get_common_cli_args(true, true, false, config))
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

pub fn execute_contract(
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
    .args(get_common_cli_args(true, true, false, config))
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

pub fn migrate_contract(
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
    .args(get_common_cli_args(true, true, false, config))
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
pub fn query_tx(tx_hash: &str) -> Result<TxQueryResponse, WarpError> {
    let mut retries = 10;
    loop {
        let cmd = Command::new("archwayd")
            .args(vec!["q", "tx", tx_hash])
            .args(get_common_cli_args(
                false,
                true,
                false,
                &ProjectConfig::empty(),
            ))
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

pub fn query_contract_smart(
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
        .args(get_common_cli_args(false, true, false, config))
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
