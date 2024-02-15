use std::path::PathBuf;

use serde_json::Value;

use crate::archway::keys_show::KeysShowResponse;
use crate::archway::tx_query::TxQueryResponse;
use crate::commands::config::NetworkConfig;
use crate::utils::project_config::{Network, ProjectConfig};
use crate::WarpError;

pub trait ChainProfile {
    fn get_profile_name(&self) -> String;
    fn get_common_cli_args<'a, 'b>(
        &self,
        tx: bool,
        network: bool,
        store: bool,
        config: &'b ProjectConfig,
    ) -> Vec<String>;
    fn get_key_info(
        &self,
        account_id: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<KeysShowResponse, WarpError>;
    fn store_contract(
        &self,
        contract: &str,
        from: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError>;
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
    ) -> Result<TxQueryResponse, WarpError>;
    fn execute_contract(
        &self,
        contract_address: &str,
        msg: &str,
        from: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError>;
    fn migrate_contract(
        &self,
        contract_address: &str,
        code_id: &str,
        from: &str,
        migrate_msg: &str,
        password: Option<&str>,
        config: &ProjectConfig,
    ) -> Result<TxQueryResponse, WarpError>;
    fn query_tx(&self, tx_hash: &str, config: &ProjectConfig)
        -> Result<TxQueryResponse, WarpError>;
    fn query_contract_smart(
        &self,
        contract: &str,
        query: &str,
        config: &ProjectConfig,
    ) -> Result<Value, WarpError>;
    fn init_project(&self, dir: &PathBuf) -> Result<(), WarpError>;
    fn new_contract(
        &self,
        contract_name: &str,
        contract_dir: &PathBuf,
        project_root: &PathBuf,
    ) -> Result<(), WarpError>;

    fn get_node_docker_command(&self, container: Option<String>, config: &ProjectConfig) -> String;

    fn network_params(&self, network_config: &NetworkConfig) -> Network;
    fn get_initialized_address(&self, tx: &TxQueryResponse) -> String;
    fn init_frontend(&self, dir: &PathBuf) -> Result<(), WarpError>;
}
