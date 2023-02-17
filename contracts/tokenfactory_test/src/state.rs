use cosmwasm_schema::cw_serde;
// use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

// TODO: allow multiple denoms to be managed by 1 contract

#[cw_serde]
pub struct Config {
    pub core_address: String
}

pub const STATE: Item<Config> = Item::new("config");