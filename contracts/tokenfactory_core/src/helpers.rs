use cosmwasm_std::Addr;

use crate::{state::Config, ContractError};

pub fn is_whitelisted(state: Config, sender: Addr) -> Result<(), ContractError> {
    if !state.allowed_mint_addresses.contains(&sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn is_contract_manager(state: Config, sender: Addr) -> Result<(), ContractError> {
    if !state.manager.eq(&sender.to_string()) {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}
