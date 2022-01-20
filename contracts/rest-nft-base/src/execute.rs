use cosmwasm_std::{coins, BankMsg, Deps, DepsMut, Env, MessageInfo, Response, Uint128};
use std::convert::From;

use cw721_base::state::TokenInfo;
use cw721_base::MintMsg;
use rest_nft::state::{Trait, Extension, Metadata, RestNFTContract};

use crate::error::ContractError;
use crate::state::{Config, Sales, CONFIG, SALES};

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();

    let token = cw721_contract.tokens.load(deps.storage, &token_id)?;
    // validate send permissions
    _check_can_send(&cw721_contract, deps.as_ref(), &env, &info, &token)?;

    cw721_contract.tokens.remove(deps.storage, &token_id)?;
    cw721_contract
        .token_count
        .update(deps.storage, |count| -> Result<u64, ContractError> {
            Ok(count - 1)
        })?;

    Ok(Response::new()
        .add_attribute("action", "burn")
        .add_attribute("token_id", token_id))
}

// Copied private cw721 check here
fn _check_can_send<T>(
    cw721_contract: &RestNFTContract,
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token: &TokenInfo<T>,
) -> Result<(), ContractError> {
    // owner can send
    if token.owner == info.sender {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
    let op = cw721_contract
        .operators
        .may_load(deps.storage, (&token.owner, &info.sender))?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(ContractError::Unauthorized {})
            } else {
                Ok(())
            }
        }
        None => Err(ContractError::Unauthorized {}),
    }
}

pub fn execute_update(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    token_uri: Option<String>,
    extension: Extension,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    let config = CONFIG.load(deps.storage)?;

    if config.frozen {
        return Err(ContractError::ContractFrozen {});
    }

    cw721_contract
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(mut token_info) => {
                token_info.token_uri = token_uri;
                token_info.extension = extension;
                Ok(token_info)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attribute("action", "update")
        .add_attribute("token_id", token_id))
}

pub fn execute_freeze(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.frozen = true;
            Ok(config)
        },
    )?;

    Ok(Response::new().add_attribute("action", "freeze"))
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_msg: MintMsg<Extension>,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let current_count = cw721_contract.token_count(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    if config.token_supply.is_some() && current_count >= config.token_supply.unwrap() {
        return Err(ContractError::MaxTokenSupply {});
    }

    let response = cw721_contract.mint(deps, env, info, mint_msg)?;
    Ok(response)
}

pub fn execute_set_level(
    deps: DepsMut,
    info: MessageInfo,
    token_id: String,
    level: String,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    cw721_contract
        .tokens
        .update(deps.storage, &token_id, |token| match token {
            Some(token_info) => {
                let mut update_token = token_info;
                let metadata = update_token.extension.clone().unwrap();
                let mut new_attributes = metadata.attributes.clone().unwrap();
                let mut count = 0u32;
                loop {
                    let mut new_attribute = new_attributes[count as usize].clone();
                    if new_attribute.trait_type == "level" {
                        new_attribute = Trait {
                            value: level,
                            trait_type: "level".to_string(),
                            display_type: Some("null".to_string())
                        };
                        new_attributes[count as usize] = new_attribute;
                        break;
                    }
                    count += 1;

                }
                let new_metadata = Metadata {
                    image: metadata.image,
                    image_data: metadata.image_data,
                    external_url: metadata.external_url,
                    description: metadata.description,
                    name: metadata.name,
                    attributes: Some(new_attributes),
                    background_color: metadata.background_color,
                    animation_url: metadata.animation_url,
                    youtube_url: metadata.youtube_url,
                };
                update_token.extension = Extension::Some(new_metadata);
                Ok(update_token)
            }
            None => return Err(ContractError::TokenNotFound {}),
        })?;

    Ok(Response::new()
        .add_attribute("action", "set_level")
        .add_attribute("sender", info.sender)
        .add_attribute("level", level)
        .add_attribute("token_id", token_id))
}

pub fn execute_set_buy_amount(
    deps: DepsMut,
    info: MessageInfo,
    buy_amount: u64,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.buy_amount = buy_amount;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "set_mint_amount")
        .add_attribute("sender", info.sender)
        .add_attribute("amount", buy_amount.to_string()))
}

pub fn execute_set_available(
    deps: DepsMut,
    info: MessageInfo,
    available: bool,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(
        deps.storage,
        |mut config| -> Result<Config, ContractError> {
            config.available = available;
            Ok(config)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "set_available")
        .add_attribute("sender", info.sender)
        .add_attribute("available", available.to_string()))
}

pub fn execute_buy(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;
    let sales = SALES.load(deps.storage)?;
    let token_id = sales.count + 1;
    let config = CONFIG.load(deps.storage)?;

    let buy_amount = config.buy_amount;

    if config.available != true {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(coins) = info.funds.first() {
        if coins.denom != "uusd" || coins.amount != Uint128::from(buy_amount) {
            return Err(ContractError::Funds {});
        }
    } else {
        return Err(ContractError::Funds {});
    }

    let message = BankMsg::Send {
        to_address: minter.to_string(),
        amount: coins(buy_amount.into(), "uusd"),
    };

    let mut token = cw721_contract
        .tokens
        .load(deps.storage, &token_id.to_string())?;
    token.owner = deps.api.addr_validate(&recipient)?;
    cw721_contract
        .tokens
        .save(deps.storage, &token_id.to_string(), &token)?;

    cw721_contract
        .token_count
        .update(deps.storage, |count| -> Result<u64, ContractError> {
            Ok(count + 1)
        })?;

    SALES.update(deps.storage, |mut sales| -> Result<Sales, ContractError> {
        sales.count = token_id;
        Ok(sales)
    })?;

    Ok(Response::new()
        .add_message(message)
        .add_attribute("action", "buy")
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient)
        .add_attribute("token_id", token_id.to_string()))
}

pub fn execute_set_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: String,
) -> Result<Response, ContractError> {
    let cw721_contract = RestNFTContract::default();
    let minter = cw721_contract.minter.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    let new_minter = deps.api.addr_validate(&new_minter)?;
    cw721_contract.minter.save(deps.storage, &new_minter)?;

    Ok(Response::new().add_attribute("action", "set_minter"))
}
