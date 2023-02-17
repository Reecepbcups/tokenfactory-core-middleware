use cosmwasm_schema::{cw_serde, QueryResponses};
use tokenfactory_core::msg::Denom;

#[cw_serde]
pub struct InstantiateMsg {
    // Assuming we handle all the denoms in 1 contract, we put that here.
    pub core_factory_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    MintTokens {
        // core_factory_address: String, // handled in state.rs now
        denoms: Vec<Denom>,
        to_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
