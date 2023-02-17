use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

// TODO: allow multiple denoms to be managed by 1 contract

#[cw_serde]
pub struct Config {
    pub manager: String,
    pub allowed_mint_addresses: Vec<String>,
    pub denom_name: String,
}

pub const STATE: Item<Config> = Item::new("config");
