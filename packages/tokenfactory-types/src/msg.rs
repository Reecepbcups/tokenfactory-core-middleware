use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, CosmosMsg, CustomMsg, StdResult, Uint128};

#[cw_serde]
pub struct Denom {
    // future: add an optional Name so its more human readable for contract authors
    pub full_denom: String,
    // this is only used in the execute_mint message to make it easier
    pub amount: Option<Uint128>,
}


#[cw_serde]
pub enum ExecuteMsg {
    // Anyone
    Burn {},

    // If an address is in the whitelist, we remove. if its not, we add it
    // Could be a DAO, normal contract, or CW4
    // Future: should we specify what name/denom an address can mint?
    ModifyWhitelist { addresses: Vec<String> },

    // Mints actual tokens to an address (only whitelisted addresses can do this)
    Mint { address: String, denom: Vec<Denom> },

    // Only the manager can do this
    TransferAdmin { denom: String, new_address: String },
}