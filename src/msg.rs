use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub initial_price: Uint128,
    pub royalty_address: String, 
}


#[cw_serde]
pub enum ExecuteMsg {
    /// Buy a plot at the given coordinates.
    Buy { coordinates: (i32, i32) },
    /// Claim funds from the treasury based on the user's points.
    Claim {},
}


