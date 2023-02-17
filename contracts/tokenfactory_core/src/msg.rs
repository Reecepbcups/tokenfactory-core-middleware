use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    // the manager of the contract is the one who can transfer the admin to another address
    // Typically this should be a multisig or a DAO (https://daodao.zone/)
    // Default is the contract initializer
    pub manager: Option<String>,
    pub allowed_mint_addresses: Vec<String>,

    // TODO: allow multiple, and to add / remove later from contract manager
    pub denom_name: String, // ex: factory/juno1xxx/test
}

#[cw_serde]
pub enum ExecuteMsg {
    // Anyone
    Burn {},

    // Add/Remove an address for who/what can Mint tokens
    // Could also be a DAO or CW4 contract
    AddWhitelistedMintAddress { address: String },
    RemoveWhitelistedMintAddress { address: String },

    // Mints actual tokens to an address (only whitelisted addresses can do this)
    Mint { amount: Uint128, address: String },

    // Only the manager can do this
    ChangeTokenAdmin { address: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},
}
