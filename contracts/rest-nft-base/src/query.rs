use cosmwasm_std::{Deps, StdResult};

use crate::state::{Config, CONFIG, Sales, SALES};

pub fn query_config(deps: Deps) -> StdResult<Config> {
    CONFIG.load(deps.storage)
}

pub fn query_sales(deps: Deps) -> StdResult<Sales> {
    SALES.load(deps.storage)
}

pub fn query_frozen(deps: Deps) -> StdResult<bool> {
    let config = CONFIG.load(deps.storage)?;
    Ok(config.frozen)
}
