#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod router {
    use openbrush::traits::Storage;
    use uniswap_v2::{
        impls::router::*,
        traits::router::*,
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct RouterContract {
        #[storage_field]
        router: data::Data,
    }

    impl Router for RouterContract {}

    impl RouterContract {
        #[ink(constructor)]
        pub fn new(factory: AccountId, wnative: AccountId) -> Self {
            let mut instance = Self::default();
            instance.router.factory = factory;
            instance.router.wnative = wnative;
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn initialize_works() {
            let factory = AccountId::from([0x03; 32]);
            let wnative = AccountId::from([0x04; 32]);
            let router = RouterContract::new(factory, wnative);
            assert_eq!(router.factory(), factory);
            assert_eq!(router.wnative(), wnative);
        }
    }
}
