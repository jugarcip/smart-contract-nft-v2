use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The maximum allowed number of tokens
    pub token_supply: Option<u64>,
    pub frozen: bool,
    pub buy_amount: u64,
    pub available: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Sales {
    pub count: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SALES: Item<Sales> = Item::new("sales");
