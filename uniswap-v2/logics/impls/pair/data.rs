use crate::traits::types::WrappedU256;
use openbrush::traits::{
    AccountId,
    Balance,
    Timestamp,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
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
    pub lock: bool,
}
