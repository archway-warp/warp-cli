pub const MSG_FILE: &'static str = "use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::CanonicalAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: CanonicalAddr,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = \"snake_case\")]
pub enum ExecuteMsg {
    ChangeOwner { addr: CanonicalAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = \"snake_case\")]
pub enum QueryMsg {
    // GetOwner returns the current owner of the contract
    GetOwner {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
    pub owner: CanonicalAddr,
}";
