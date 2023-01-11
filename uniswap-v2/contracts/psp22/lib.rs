#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod token {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String,
        },
    };

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct MyPSP22 {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl PSP22 for MyPSP22 {}

    impl psp22::Internal for MyPSP22 {
        fn _emit_transfer_event(
            &self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            amount: Balance,
        ) {
            self.env().emit_event(Transfer {
                from,
                to,
                value: amount,
            });
        }

        fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, amount: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
        }
    }

    impl PSP22Metadata for MyPSP22 {}

    impl MyPSP22 {
        #[ink(constructor)]
        pub fn new(
            total_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
        ) -> Self {
            let mut instance = Self::default();
            instance.metadata.name = name;
            instance.metadata.symbol = symbol;
            instance.metadata.decimals = decimals;
            instance
                ._mint_to(instance.env().caller(), total_supply)
                .expect("Should mint");
            instance
        }
        #[ink(message)]
        /// Permissionless mint, test purpose only. DO NOT use for production.
        /// Users can test our uniswap v2 demo on Shibuya by minting it by themselves.
        pub fn mint(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            self._mint_to(account, amount)
        }
    }
}
