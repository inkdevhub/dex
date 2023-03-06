use openbrush::traits::{
    AccountId,
    ZERO_ADDRESS,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub factory: AccountId,
    pub wnative: AccountId,
    pub pair_code_hash: Hash,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: ZERO_ADDRESS.into(),
            wnative: ZERO_ADDRESS.into(),
        }
    }
}
