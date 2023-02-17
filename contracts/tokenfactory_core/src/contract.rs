#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{BankMsg, Coin, DepsMut, Env, MessageInfo, Response, to_binary, StdResult, Binary, Deps};
use cw2::set_contract_version;

use tokenfactory_types::msg::Denom as Denom;

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
        if !d.full_denom.starts_with("factory/") {
            return Err(ContractError::InvalidDenom {
                denom: d.full_denom.clone(),
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

    let denom =
        state
            .denoms
            .iter()
            .find(|d| d.full_denom == denom)
            .ok_or(ContractError::InvalidDenom {
                denom,
                message: "Denom not found in state".to_string(),
            })?;

    let msg = TokenMsg::ChangeAdmin {
        denom: denom.full_denom.to_string(),
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
    denoms: Vec<Denom>,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;

    is_whitelisted(state.clone(), info.sender)?;

    if denoms.is_empty() {
        return Err(ContractError::InvalidDenom {
            denom: "denoms".to_string(),
            message: "denoms cannot be empty on mint".to_string(),
        });
    }

    for d in denoms.clone() {
        // check if the denom is in the state, if not, we can not mint it to a user.
        // find the full denom OR name if it is set
        let tmp_denom = state
            .denoms
            .iter()
            .find(|denom| denom.full_denom == d.full_denom)
            .ok_or(ContractError::InvalidDenom {
                denom: d.full_denom,
                message: "Denom not found in state".to_string(),
            })?;

        // ensure denom has amount set, else we can not send
        if tmp_denom.amount.is_none() {
            return Err(ContractError::InvalidDenom {
                denom: tmp_denom.full_denom.to_string(),
                message: "Denom does not have amount set".to_string(),
            });
        }
    }

    // create the send messages
    let msgs: Vec<TokenMsg> = denoms
        .iter()
        .map(|d| TokenMsg::MintTokens {
            denom: d.full_denom.clone(),
            amount: d.amount.unwrap(),
            mint_to_address: address.to_string(),
        })
        .collect();

    // get all full_denom & amounts as a string in the format [{full_denom: amount}, ], ex: [{uusd: 1000000}, {ujuno: 1000000}]
    let output = denoms
        .iter()
        .map(|d| format!("{{{}: {}}}", d.full_denom, d.amount.unwrap()))
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

    // the difference between the funds sent and the funds to send back
    let factory_denoms: Vec<Coin> = info
        .funds
        .iter()
        .filter(|coin| state.denoms.iter().any(|d| d.full_denom == coin.denom))
        .cloned()
        .collect();

    let send_back = info
        .funds
        .iter()
        .filter(|coin| !factory_denoms.contains(coin))
        .cloned()
        .collect();

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

// pub fn execute_redeem_balance(
//     deps: DepsMut,
//     info: MessageInfo,
//     env: Env,
//     cw20_msg: Cw20ReceiveMsg,
// ) -> Result<Response, ContractError> {
//     let cw20_contract = info.sender.to_string();
//     let state = STATE.load(deps.storage)?;

//     if cw20_contract != state.cw20_address {
//         return Err(ContractError::InvalidCW20Address {});
//     }

//     let contract_balance = deps.querier.query_all_balances(env.contract.address)?;
//     let contract_balance = contract_balance
//         .iter()
//         .find(|c| c.denom == state.tf_denom)
//         .unwrap();

//     if contract_balance.amount < cw20_msg.amount {
//         return Err(ContractError::OutOfFunds {
//             request: cw20_msg.amount,
//             amount: contract_balance.amount,
//         });
//     }

//     // Send our token-factory balance to the sender of the CW20 tokens for the exchange
//     let bank_msg = BankMsg::Send {
//         to_address: cw20_msg.sender.clone(),
//         amount: vec![Coin {
//             denom: state.tf_denom,
//             amount: cw20_msg.amount,
//         }],
//     };

//     // Burn the CW20 since it is in our possession now
//     let cw20_burn = cw20::Cw20ExecuteMsg::Burn {
//         amount: cw20_msg.amount,
//     };
//     let cw20_burn_msg: WasmMsg = WasmMsg::Execute {
//         contract_addr: cw20_contract,
//         msg: to_binary(&cw20_burn)?,
//         funds: vec![],
//     };

//     Ok(Response::new()
//         .add_attribute("method", "redeem_balannce")
//         .add_message(cw20_burn_msg)
//         .add_message(bank_msg))
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => {
            let state = STATE.load(deps.storage)?;
            to_binary(&state)
        }
    }
}
