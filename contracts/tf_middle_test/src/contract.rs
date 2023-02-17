#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

// version info for migration info
// const CONTRACT_NAME: &str = "crates.io:tokenfactory-test";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // if !msg.denom_name.starts_with("factory/") {
    //     return Err(ContractError::InvalidDenom {
    //         denom: msg.denom_name,
    //         message: "Denom must start with 'factory/'".to_string(),
    //     });
    // }

    // let manager = deps
    //     .api
    //     .addr_validate(&msg.manager.unwrap_or_else(|| _info.sender.to_string()))?;

    // let config = Config {
    //     manager: manager.to_string(),
    //     allowed_mint_addresses: msg.allowed_mint_addresses,
    //     denom_name: msg.denom_name,
    // };
    // STATE.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

use tokenfactory_core::msg::ExecuteMsg::Mint;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // put tf address in instatiate msg
        ExecuteMsg::MintTokens { tf_address, address, amount } => {
                    
            let msg = WasmMsg::Execute {
                contract_addr: tf_address.to_string(),
                msg: to_binary(&Mint { amount, address })?,
                funds: vec![],
            };

            Ok(Response::new()
                .add_message(msg)
                .add_attribute("method", "execute_mint_tokens"))
        },
    }
}

// pub fn execute_transfer_admin(
//     deps: DepsMut,
//     info: MessageInfo,
//     new_addr: String,
// ) -> Result<Response, ContractError> {
//     let state = STATE.load(deps.storage)?;
//     if info.sender != state.manager {
//         return Err(ContractError::Unauthorized {});
//     }

//     let msg = TokenMsg::ChangeAdmin {
//         denom: state.denom_name,
//         new_admin_address: new_addr.to_string(),
//     };

//     Ok(Response::new()
//         .add_attribute("method", "execute_transfer_admin")
//         .add_attribute("new_admin", new_addr)
//         .add_message(msg))
// }

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

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetConfig {} => {
//             let state = STATE.load(deps.storage)?;
//             to_binary(&GetConfig {
//                 cw20_address: state.cw20_address.into_string(),
//                 tf_denom: state.tf_denom,
//                 mode: "balance".to_string(),
//             })
//         }
//     }
// }
