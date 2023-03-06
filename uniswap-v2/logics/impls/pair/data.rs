use crate::traits::types::WrappedU256;
use openbrush::traits::{
    AccountId,
    Balance,
    Timestamp,
    ZERO_ADDRESS,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub factory: AccountId,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub reserve_0: Balance,
    pub reserve_1: Balance,
    pub block_timestamp_last: Timestamp,
    pub price_0_cumulative_last: WrappedU256,
    pub price_1_cumulative_last: WrappedU256,
    pub k_last: WrappedU256,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: ZERO_ADDRESS.into(),
            token_0: ZERO_ADDRESS.into(),
            token_1: ZERO_ADDRESS.into(),
            reserve_0: 0,
            reserve_1: 0,
            block_timestamp_last: 0,
            price_0_cumulative_last: Default::default(),
            price_1_cumulative_last: Default::default(),
            k_last: Default::default(),
        }
    }
}
