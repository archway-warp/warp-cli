use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateFeesResponse {
    #[serde(rename = "gas_unit_price")]
    pub gas_unit_price: GasUnitPrice,
    #[serde(rename = "estimated_fee")]
    pub estimated_fee: Vec<EstimatedFee>,
}

impl EstimateFeesResponse {
    pub fn get_gas_price(&self) -> String {
        format!(
            "{}{}",
            &self.gas_unit_price.amount, &self.gas_unit_price.denom
        )
    }

    pub fn get_fee(&self) -> String {
        let fee = self.estimated_fee.first();
        let fee = fee.unwrap();
        format!("{}{}", &fee.amount, &fee.denom)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasUnitPrice {
    pub denom: String,
    pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimatedFee {
    pub denom: String,
    pub amount: String,
}
