#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::{is_contract_manager, is_whitelisted};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, STATE, WHITELIST_ADDRESSES};

use token_bindings::{TokenFactoryMsg, TokenMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:tokenfactory-middleware";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    for d in msg.denoms.iter() {
        if !d.starts_with("factory/") {
            return Err(ContractError::InvalidDenom {
                denom: d.clone(),
                message: "Denom must start with 'factory/'".to_string(),
            });
        }
    }

    let manager = deps
        .api
        .addr_validate(&msg.manager.unwrap_or_else(|| _info.sender.to_string()))?;

    let config = Config {
        manager: manager.to_string(),
        allowed_mint_addresses: msg.allowed_mint_addresses,
        denoms: msg.denoms,
    };
    STATE.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match msg {
        // Permissionless
        ExecuteMsg::Burn {} => execute_burn(deps, env, info),

        // Contract whitelist only
        ExecuteMsg::Mint { address, denom } => execute_mint(deps, info, address, denom),
        ExecuteMsg::TransferAdmin { denom, new_address } => {
            execute_transfer_admin(deps, info, denom, new_address)
        }

        // Contract manager only
        ExecuteMsg::ModifyWhitelist { addresses } => {
            let state = STATE.load(deps.storage)?;
            is_contract_manager(state, info.sender)?;

            // loop through all addresses, and see if they are in the whitelist. If so, remove them, if not, add them
            for address in addresses.iter() {
                let addr = deps.api.addr_validate(address)?;

                if WHITELIST_ADDRESSES
                    .may_load(deps.storage, addr.to_string())?
                    .is_some()
                {
                    WHITELIST_ADDRESSES.remove(deps.storage, addr.to_string());
                } else {
                    WHITELIST_ADDRESSES.save(deps.storage, addr.to_string(), &true)?;
                }
            }

            Ok(Response::new())
        }
    }
}

pub fn execute_transfer_admin(
    deps: DepsMut,
    info: MessageInfo,
    denom: String,
    new_addr: String,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    is_contract_manager(state.clone(), info.sender)?;

    let denom = state.denoms.iter().find(|d| d.to_string() == denom).ok_or(
        ContractError::InvalidDenom {
            denom,
            message: "Denom not found in state".to_string(),
        },
    )?;

    let msg = TokenMsg::ChangeAdmin {
        denom: denom.to_string(),
        new_admin_address: new_addr.to_string(),
    };

    Ok(Response::new()
        .add_attribute("method", "execute_transfer_admin")
        .add_attribute("new_admin", new_addr)
        .add_message(msg))
}

pub fn execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    denoms: Vec<Coin>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    is_whitelisted(state.clone(), info.sender)?;

    if denoms.is_empty() {
        return Err(ContractError::InvalidDenom {
            denom: "denoms".to_string(),
            message: "denoms cannot be empty on mint".to_string(),
        });
    }

    // create the send messages
    let msgs: Vec<TokenMsg> = denoms
        .iter()
        .filter(|d| denoms.iter().any(|d2| d2.denom == d.denom))
        .map(|d| TokenMsg::MintTokens {
            denom: d.denom.clone(),
            amount: d.amount,
            mint_to_address: address.to_string(),
        })
        .collect();

    // get all full_denom & amounts as a string in the format [{full_denom: amount}, ], ex: [{uusd: 1000000}, {ujuno: 1000000}]
    let output = denoms
        .iter()
        .map(|d| format!("{{{}: {}}}", d.denom, d.amount))
        .collect::<Vec<String>>()
        .join(", ");

    Ok(Response::new()
        .add_attribute("method", "execute_mint")
        .add_attribute("to_address", address)
        .add_attribute("denoms", output)
        .add_messages(msgs))
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    // Anyone can burn funds since they have to send them in.
    if info.funds.is_empty() {
        return Err(ContractError::InvalidFunds {});
    }

    let state = STATE.load(deps.storage)?;

    let (factory_denoms, send_back): (Vec<Coin>, Vec<Coin>) = info
        .funds
        .iter()
        .cloned()
        .partition(|coin| state.denoms.iter().any(|d| d.to_string() == coin.denom));

    let burn_msgs: Vec<TokenMsg> = factory_denoms
        .iter()
        .map(|coin| TokenMsg::BurnTokens {
            denom: coin.denom.clone(),
            amount: coin.amount,
            burn_from_address: env.contract.address.to_string(),
        })
        .collect();

    let bank_return_msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: send_back,
    };

    Ok(Response::new()
        .add_attribute("method", "execute_burn")
        .add_message(bank_return_msg)
        .add_messages(burn_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state)
        }
    }
}
