#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod wnative {
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
    use uniswap_v2::impls::wnative::*;

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
    pub struct WnativeContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl PSP22 for WnativeContract {}

    impl psp22::Internal for WnativeContract {
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

    impl Wnative for WnativeContract {}

    impl PSP22Metadata for WnativeContract {}

    impl WnativeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.metadata.name = Some(String::from("Wrapped Native"));
            instance.metadata.symbol = Some(String::from("WNATIVE"));
            instance.metadata.decimals = 18;
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_works() {
            let wnative_contract = WnativeContract::new();
            assert_eq!(
                wnative_contract.metadata.name,
                Some(String::from("Wrapped Native"))
            );
            assert_eq!(
                wnative_contract.metadata.symbol,
                Some(String::from("WNATIVE"))
            );
        }

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(0);
            assert_eq!(deposit(&mut wnative_contract, 1000), Ok(()));
            let balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(balance, 1000, "balance not correct!");
            let native_balance: Balance = wnative_contract.env().balance();
            assert_eq!(native_balance, 1000, "native balance not correct!");
        }

        #[ink::test]
        fn test_withdraw() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(1000);
            assert_eq!(get_balance(wnative_contract.env().account_id()), 1000);
            assert_eq!(
                wnative_contract._mint_to(accounts.alice, 1000),
                Ok(()),
                "mint failed"
            );
            let wnative_balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(wnative_balance, 1000, "balance not correct!");

            let before_balance = get_balance(accounts.alice);
            assert_eq!(wnative_contract.withdraw(800), Ok(()));
            assert_eq!(
                get_balance(accounts.alice),
                800 + before_balance,
                "withdraw should refund native token"
            );
            let wnative_balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(wnative_balance, 200, "balance not correct!");
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts()
        }

        fn set_next_caller(caller: AccountId) {
            ink::env::test::set_caller::<Environment>(caller);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        /// Creates a new instance of `WnativeContract` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> WnativeContract {
            let accounts = default_accounts();
            set_next_caller(accounts.alice);
            set_balance(contract_id(), initial_balance);
            WnativeContract::new()
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        fn deposit(contract: &mut WnativeContract, amount: Balance) -> Result<(), PSP22Error> {
            let sender = ink::env::caller::<ink::env::DefaultEnvironment>();
            let contract_id = contract.env().account_id();
            let sender_balance = get_balance(sender);
            let contract_balance = get_balance(contract_id);
            // ↓ doesn't work, is upstream issue: https://github.com/paritytech/ink/issues/1117
            // set_balance(sender, sender_balance - amount);
            set_balance(
                sender,
                if sender_balance > amount {
                    sender_balance - amount
                } else {
                    0
                },
            );
            set_balance(contract_id, contract_balance + amount);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(amount);
            contract.deposit()
        }
    }
}
