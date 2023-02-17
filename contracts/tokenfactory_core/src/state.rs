use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

use crate::msg::Denom;

#[cw_serde]
pub struct Config {
    pub manager: String,
    pub allowed_mint_addresses: Vec<String>,
    pub denoms: Vec<Denom>,
}

pub const STATE: Item<Config> = Item::new("config");

pub const WHITELIST_ADDRESSES: Map<String, bool> = Map::new("whitelist_addresses");
