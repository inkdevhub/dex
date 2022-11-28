#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod router {
    use ink_storage::traits::SpreadAllocate;
    use openbrush::traits::Storage;
    use uniswap_v2::{
        impls::router::*,
        traits::router::*,
    };

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct RouterContract {
        #[storage_field]
        router: data::Data,
    }

    impl Router for RouterContract {}

    impl RouterContract {
        #[ink(constructor)]
        pub fn new(factory: AccountId, wnative: AccountId, pair_code_hash: Hash) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.router.factory = factory;
                instance.router.wnative = wnative;
                instance.router.pair_code_hash = pair_code_hash;
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_env::AccountId;

        #[ink_lang::test]
        fn initialize_works() {
            let factory = AccountId::from([0x03; 32]);
            let wnative = AccountId::from([0x04; 32]);
            let pair_code_hash = Hash::default();
            let router = RouterContract::new(factory, wnative, pair_code_hash);
            assert_eq!(router.factory(), factory);
            assert_eq!(router.wnative(), wnative);
        }
    }
}
