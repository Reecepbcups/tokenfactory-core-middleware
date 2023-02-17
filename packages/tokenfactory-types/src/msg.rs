use cosmwasm_schema::cw_serde;

use cosmwasm_std::Coin;

#[cw_serde]
pub enum ExecuteMsg {
    // Anyone
    Burn {},

    // If an address is in the whitelist, we remove. if its not, we add it
    // Could be a DAO, normal contract, or CW4
    // Future: should we specify what name/denom an address can mint?
    ModifyWhitelist { addresses: Vec<String> },

    // Mints actual tokens to an address (only whitelisted addresses can do this)
    Mint { address: String, denom: Vec<Coin> },

    // Only the manager can do this
    TransferAdmin { denom: String, new_address: String },
}
