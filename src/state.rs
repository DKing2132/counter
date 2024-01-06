use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub const PLOTS: Map<(i32, i32), Plot> = Map::new("plots");
pub const TREASURY: Item<Uint128> = Item::new("treasury");
pub const ROYALTY: Item<Uint128> = Item::new("royalty");
pub const POINTS: Map<Addr, Uint128> = Map::new("points");
pub const TOTAL_POINTS: Item<Uint128> = Item::new("total_points");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Plot {
    pub coordinates: (i32, i32),
    pub price: Uint128,
    pub owner: Addr,
}

