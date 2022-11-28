#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod pair {
    use ink_lang::codegen::{
        EmitEvent,
        Env,
    };
    use ink_prelude::vec::Vec;
    use ink_storage::traits::SpreadAllocate;
    use openbrush::{
        contracts::{
            ownable::*,
            psp22::*,
            reentrancy_guard,
        },
        traits::Storage,
    };
    use uniswap_v2::{
        ensure,
        impls::pair::*,
        traits::pair::*,
    };

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0_in: Balance,
        pub amount_1_in: Balance,
        pub amount_0_out: Balance,
        pub amount_1_out: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Sync {
        reserve_0: Balance,
        reserve_1: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate, Storage)]
    pub struct PairContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        pair: data::Data,
    }

    impl PSP22 for PairContract {
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let allowance = self._allowance(&from, &caller);

            // In uniswapv2 max allowance never decrease
            if allowance != u128::MAX {
                ensure!(allowance >= value, PSP22Error::InsufficientAllowance);
                self._approve_from_to(from, caller, allowance - value)?;
            }
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }
    }

    impl psp22::Internal for PairContract {
        // in uniswapv2 no check for zero account
        fn _mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut new_balance = self._balance_of(&account);
            new_balance += amount;
            self.psp22.balances.insert(&account, &new_balance);
            self.psp22.supply += amount;
            self._emit_transfer_event(None, Some(account), amount);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut from_balance = self._balance_of(&account);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            from_balance -= amount;
            self.psp22.balances.insert(&account, &from_balance);
            self.psp22.supply -= amount;
            self._emit_transfer_event(Some(account), None, amount);
            Ok(())
        }

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self.psp22.allowances.insert(&(&owner, &spender), &amount);
            self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from_balance = self._balance_of(&from);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            self.psp22.balances.insert(&from, &(from_balance - amount));
            let to_balance = self._balance_of(&to);
            self.psp22.balances.insert(&to, &(to_balance + amount));

            self._emit_transfer_event(Some(from), Some(to), amount);
            Ok(())
        }

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
    }

    impl Ownable for PairContract {}

    impl pair::Internal for PairContract {
        fn _emit_mint_event(&self, sender: AccountId, amount_0: Balance, amount_1: Balance) {
            self.env().emit_event(Mint {
                sender,
                amount_0,
                amount_1,
            })
        }

        fn _emit_burn_event(
            &self,
            sender: AccountId,
            amount_0: Balance,
            amount_1: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Burn {
                sender,
                amount_0,
                amount_1,
                to,
            })
        }

        fn _emit_swap_event(
            &self,
            sender: AccountId,
            amount_0_in: Balance,
            amount_1_in: Balance,
            amount_0_out: Balance,
            amount_1_out: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Swap {
                sender,
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            })
        }

        fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance) {
            self.env().emit_event(Sync {
                reserve_0,
                reserve_1,
            })
        }
    }

    impl Pair for PairContract {}

    impl PairContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|instance: &mut Self| {
                let caller = instance.env().caller();
                instance._init_with_owner(caller);
                instance.pair.factory = caller;
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use ink_env::AccountId;

        use super::*;

        #[ink_lang::test]
        fn initialize_works() {
            let mut pair = PairContract::new();
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            assert_eq!(pair.initialize(token_0, token_1), Ok(()));
        }
    }
}
