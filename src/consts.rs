pub const MSG_FILE: &str = "use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub message: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ChangeOwner { addr: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetOwner returns the current owner of the contract
    #[returns(OwnerResponse)]
    GetOwner {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[cw_serde]
pub struct MigrateMsg {}
";
