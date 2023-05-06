use std::{
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    archway::{keys_show::KeysShowResponse, tx_query::TxQueryResponse},
    error::WarpError,
};

use super::{command_util::CommandWithInput, project_config::ProjectConfig};

fn get_common_cli_args<'a>(
    tx: bool,
    network: bool,
    store: bool,
    config: &'a ProjectConfig,
) -> Vec<&'a str> {
    let mut args = vec!["--output", "json"];
    if network {
        args.push("--chain-id");
        args.push(&config.network.chain_id);
        args.push("--node");
        args.push(&config.network.rpc_url);
    }
    if tx {
        let mut tx_args = vec![
            "--gas",
            if store { "3200000" } else { "1000000" },
            "-y",
            "-b",
            "sync",
            "--fees",
            "8000uconst",
        ];
        args.append(&mut tx_args);
    }
    args
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

// TODO: Make this ugly thing go away once a better solution is confirmed to be working
pub fn query_tx(tx_hash: &str) -> Result<TxQueryResponse, WarpError> {
    let mut retries = 7;
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
            std::thread::sleep(Duration::from_millis(1200));
            continue;
        }
        let response: TxQueryResponse = serde_json::from_slice(tx.as_slice())?;
        if response.code != 0 {
            return Err(WarpError::TxFailed(response.txhash, response.raw_log));
        }
        return Ok(response);
    }
}
