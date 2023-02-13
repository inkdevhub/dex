use ink::{
    prelude::vec::Vec,
    primitives::Hash,
};
use openbrush::{
    storage::Mapping,
    traits::{AccountId, ZERO_ADDRESS}
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub fee_to: AccountId,
    pub fee_to_setter: AccountId,
    pub get_pair: Mapping<(AccountId, AccountId), AccountId>,
    pub all_pairs: Vec<AccountId>,
    pub pair_contract_code_hash: Hash,
}


impl Default for Data {
    fn default() -> Self {
        Self {
            fee_to: ZERO_ADDRESS.into(),
            fee_to_setter: ZERO_ADDRESS.into(),
            get_pair: Default::default(),
            all_pairs: Vec::new(),
            pair_contract_code_hash: Default::default(),
        }
    }
}