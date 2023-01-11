use ink::{
    prelude::vec::Vec,
    primitives::Hash,
};
use openbrush::{
    storage::Mapping,
    traits::AccountId,
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub fee_to: AccountId,
    pub fee_to_setter: AccountId,
    pub get_pair: Mapping<(AccountId, AccountId), AccountId>,
    pub all_pairs: Vec<AccountId>,
    pub pair_contract_code_hash: Hash,
}
