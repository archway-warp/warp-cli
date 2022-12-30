use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeysShowResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub address: String,
    pub pubkey: String,
}
