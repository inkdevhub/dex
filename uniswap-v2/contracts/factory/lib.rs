#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod factory {
    use ink_lang::{
        codegen::{
            EmitEvent,
            Env,
        },
        ToAccountId,
    };
    use ink_storage::traits::SpreadAllocate;
    use openbrush::traits::Storage;
    use pair_contract::pair::PairContractRef;
    use uniswap_v2::{
        impls::factory::*,
        traits::factory::*,
    };

    #[ink(event)]
    pub struct PairCreated {
        #[ink(topic)]
        pub token_0: AccountId,
        #[ink(topic)]
        pub token_1: AccountId,
        pub pair: AccountId,
        pub pair_len: u64,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct FactoryContract {
        #[storage_field]
        factory: data::Data,
    }

    impl Factory for FactoryContract {}

    impl factory::Internal for FactoryContract {
        fn _instantiate_pair(&mut self, salt_bytes: &[u8]) -> AccountId {
            let pair_hash = self.factory.pair_contract_code_hash;
            let pair = PairContractRef::new()
                .endowment(0)
                .code_hash(pair_hash)
                .salt_bytes(&salt_bytes[..4])
                .instantiate()
                .unwrap();
            pair.to_account_id()
        }

        fn _emit_create_pair_event(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            pair: AccountId,
            pair_len: u64,
        ) {
            EmitEvent::<FactoryContract>::emit_event(
                self.env(),
                PairCreated {
                    token_0,
                    token_1,
                    pair,
                    pair_len,
                },
            )
        }
    }

    impl FactoryContract {
        #[ink(constructor)]
        pub fn new(fee_to_setter: AccountId, pair_code_hash: Hash) -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                instance.factory.pair_contract_code_hash = pair_code_hash;
                instance.factory.fee_to_setter = fee_to_setter;
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use ink_env::{
            test::default_accounts,
            Hash,
        };
        use openbrush::traits::AccountIdExt;

        use super::*;

        #[ink_lang::test]
        fn initialize_works() {
            let accounts = default_accounts::<ink_env::DefaultEnvironment>();
            let factory = FactoryContract::new(accounts.alice, Hash::default());
            assert!(factory.factory.fee_to.is_zero());
        }
    }
}
