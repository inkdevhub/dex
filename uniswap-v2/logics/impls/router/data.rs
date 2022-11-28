use ink_env::Hash;
use openbrush::traits::AccountId;

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub factory: AccountId,
    pub wnative: AccountId,
    pub pair_code_hash: Hash,
}
