use crate::{
    ensure,
    helpers::{
        helper::{
            get_amount_in,
            get_amount_out,
            get_amounts_in,
            get_amounts_out,
            get_reserves,
            pair_for,
            quote,
            sort_tokens,
        },
        transfer_helper::{
            safe_transfer,
            safe_transfer_from,
            safe_transfer_native,
            unwrap,
            wrap,
        },
    },
    traits::{
        factory::FactoryRef,
        pair::PairRef,
    },
};
use ink_env::CallFlags;
use ink_prelude::vec::Vec;
use openbrush::{
    contracts::traits::psp22::PSP22Ref,
    modifier_definition,
    modifiers,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

pub use crate::{
    impls::router::*,
    traits::router::*,
};

pub trait Internal {
    fn _add_liquidity(
        &self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
    ) -> Result<(Balance, Balance), RouterError>;

    fn _swap(
        &self,
        amounts: &Vec<Balance>,
        path: Vec<AccountId>,
        to: AccountId,
    ) -> Result<(), RouterError>;
}

impl<T: Storage<data::Data>> Router for T {
    default fn factory(&self) -> AccountId {
        self.data().factory
    }

    default fn wnative(&self) -> AccountId {
        self.data().wnative
    }

    #[modifiers(ensure(deadline))]
    default fn add_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance, Balance), RouterError> {
        let (amount_a, amount_b) = self._add_liquidity(
            token_a,
            token_b,
            amount_a_desired,
            amount_b_desired,
            amount_a_min,
            amount_b_min,
        )?;

        let pair_contract = pair_for(
            self.data().factory.clone().as_ref(),
            self.data().pair_code_hash.as_ref(),
            token_a,
            token_b,
        )?;

        let caller = Self::env().caller();
        safe_transfer_from(token_a, caller, pair_contract, amount_a)?;
        safe_transfer_from(token_b, caller, pair_contract, amount_b)?;

        let liquidity = PairRef::mint(&pair_contract, to)?;

        Ok((amount_a, amount_b, liquidity))
    }

    #[modifiers(ensure(deadline))]
    default fn add_liquidity_native(
        &mut self,
        token: AccountId,
        amount_token_desired: Balance,
        amount_token_min: Balance,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance, Balance), RouterError> {
        let wnative = self.data().wnative;
        let received_value = Self::env().transferred_value();
        let caller = Self::env().caller();
        let (amount, amount_native) = self._add_liquidity(
            token,
            wnative,
            amount_token_desired,
            received_value,
            amount_token_min,
            amount_native_min,
        )?;
        let pair_contract = pair_for(
            self.data().factory.clone().as_ref(),
            self.data().pair_code_hash.as_ref(),
            token,
            wnative,
        )?;

        safe_transfer_from(token, caller, pair_contract, amount)?;
        wrap(&wnative, amount_native)?;
        PSP22Ref::transfer(&wnative, pair_contract, amount_native, Vec::<u8>::new())?;
        let liquidity = PairRef::mint(&pair_contract, to)?;

        if received_value > amount_native {
            safe_transfer_native(caller, received_value - amount_native)?
        }
        Ok((amount, amount_native, liquidity))
    }

    #[modifiers(ensure(deadline))]
    default fn remove_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        liquidity: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance), RouterError> {
        let pair_contract = pair_for(
            self.data().factory.clone().as_ref(),
            self.data().pair_code_hash.as_ref(),
            token_a,
            token_b,
        )?;

        safe_transfer_from(
            pair_contract,
            Self::env().caller(),
            pair_contract,
            liquidity,
        )?;

        let (amount_0, amount_1) = PairRef::burn_builder(&pair_contract, to)
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap()?;
        let (token_0, _) = sort_tokens(token_a, token_b)?;
        let (amount_a, amount_b) = if token_a == token_0 {
            (amount_0, amount_1)
        } else {
            (amount_1, amount_0)
        };

        ensure!(amount_a >= amount_a_min, RouterError::InsufficientAAmount);
        ensure!(amount_b >= amount_b_min, RouterError::InsufficientBAmount);

        Ok((amount_a, amount_b))
    }

    #[modifiers(ensure(deadline))]
    default fn remove_liquidity_native(
        &mut self,
        token: AccountId,
        liquidity: Balance,
        amount_token_min: Balance,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance), RouterError> {
        let wnative = self.data().wnative;
        let (amount_token, amount_native) = self.remove_liquidity(
            token,
            wnative,
            liquidity,
            amount_token_min,
            amount_native_min,
            Self::env().account_id(),
            deadline,
        )?;
        safe_transfer(token, to, amount_token)?;
        unwrap(&wnative, amount_native)?;
        safe_transfer_native(to, amount_native)?;
        Ok((amount_token, amount_native))
    }

    #[modifiers(ensure(deadline))]
    default fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: Balance,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let amounts = get_amounts_out(factory_ref, pair_hash_ref, amount_in, &path)?;
        ensure!(
            amounts[amounts.len() - 1] >= amount_out_min,
            RouterError::InsufficientOutputAmount
        );
        safe_transfer_from(
            path[0],
            Self::env().caller(),
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, to)?;
        Ok(amounts)
    }

    #[modifiers(ensure(deadline))]
    default fn swap_tokens_for_exact_tokens(
        &mut self,
        amount_out: Balance,
        amount_in_max: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let amounts = get_amounts_in(factory_ref, pair_hash_ref, amount_out, &path)?;
        ensure!(
            amounts[0] <= amount_in_max,
            RouterError::ExcessiveInputAmount
        );
        safe_transfer_from(
            path[0],
            Self::env().caller(),
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, to)?;
        Ok(amounts)
    }

    #[modifiers(ensure(deadline))]
    default fn swap_exact_native_for_tokens(
        &mut self,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let received_value = Self::env().transferred_value();
        let wnative = self.data().wnative;
        ensure!(path[0] == wnative, RouterError::InvalidPath);
        let amounts = get_amounts_out(factory_ref, pair_hash_ref, received_value, &path)?;
        ensure!(
            amounts[amounts.len() - 1] >= amount_out_min,
            RouterError::InsufficientOutputAmount
        );
        wrap(&wnative, received_value)?;
        safe_transfer(
            wnative,
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, to)?;
        Ok(amounts)
    }

    #[modifiers(ensure(deadline))]
    default fn swap_tokens_for_exact_native(
        &mut self,
        amount_out: Balance,
        amount_in_max: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let wnative = self.data().wnative;
        ensure!(path[path.len() - 1] == wnative, RouterError::InvalidPath);
        let amounts = get_amounts_in(factory_ref, pair_hash_ref, amount_out, &path)?;
        ensure!(
            amounts[0] <= amount_in_max,
            RouterError::ExcessiveInputAmount
        );
        safe_transfer_from(
            path[0],
            Self::env().caller(),
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, Self::env().account_id())?;
        unwrap(&wnative, amounts[amounts.len() - 1])?;
        safe_transfer_native(to, amounts[amounts.len() - 1])?;
        Ok(amounts)
    }

    #[modifiers(ensure(deadline))]
    fn swap_exact_tokens_for_native(
        &mut self,
        amount_in: Balance,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let wnative = self.data().wnative;
        ensure!(path[path.len() - 1] == wnative, RouterError::InvalidPath);
        let amounts = get_amounts_out(factory_ref, pair_hash_ref, amount_in, &path)?;
        ensure!(
            amounts[amounts.len() - 1] >= amount_out_min,
            RouterError::InsufficientOutputAmount
        );
        safe_transfer_from(
            path[0],
            Self::env().caller(),
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, Self::env().account_id())?;
        unwrap(&wnative, amounts[amounts.len() - 1])?;
        safe_transfer_native(to, amounts[amounts.len() - 1])?;
        Ok(amounts)
    }

    #[modifiers(ensure(deadline))]
    fn swap_native_for_exact_tokens(
        &mut self,
        amount_out: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError> {
        let factory = self.data().factory;
        let pair_hash = self.data().pair_code_hash;
        let factory_ref = factory.as_ref();
        let pair_hash_ref = pair_hash.as_ref();

        let wnative = self.data().wnative;
        let received_value = Self::env().transferred_value();

        ensure!(path[0] == wnative, RouterError::InvalidPath);
        let amounts = get_amounts_in(factory_ref, pair_hash_ref, amount_out, &path)?;
        ensure!(
            amounts[0] <= received_value,
            RouterError::ExcessiveInputAmount
        );
        wrap(&wnative, amounts[0])?;
        safe_transfer(
            wnative,
            pair_for(factory_ref, pair_hash_ref, path[0], path[1])?,
            amounts[0],
        )?;
        self._swap(&amounts, path, to)?;
        if received_value > amounts[0] {
            safe_transfer_native(Self::env().caller(), received_value - amounts[0])?
        }
        Ok(amounts)
    }

    default fn quote(
        &self,
        amount_a: Balance,
        reserve_a: Balance,
        reserve_b: Balance,
    ) -> Result<Balance, RouterError> {
        Ok(quote(amount_a, reserve_a, reserve_b)?)
    }

    default fn get_amount_out(
        &self,
        amount_in: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, RouterError> {
        Ok(get_amount_out(amount_in, reserve_in, reserve_out)?)
    }

    default fn get_amount_in(
        &self,
        amount_out: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, RouterError> {
        Ok(get_amount_in(amount_out, reserve_in, reserve_out)?)
    }

    default fn get_amounts_out(
        &self,
        amount_in: Balance,
        path: Vec<AccountId>,
    ) -> Result<Vec<Balance>, RouterError> {
        Ok(get_amounts_out(
            self.data().factory.as_ref(),
            self.data().pair_code_hash.as_ref(),
            amount_in,
            &path,
        )?)
    }

    default fn get_amounts_in(
        &self,
        amount_out: Balance,
        path: Vec<AccountId>,
    ) -> Result<Vec<Balance>, RouterError> {
        Ok(get_amounts_in(
            self.data().factory.as_ref(),
            self.data().pair_code_hash.as_ref(),
            amount_out,
            &path,
        )?)
    }
}

impl<T: Storage<data::Data>> Internal for T {
    fn _add_liquidity(
        &self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
    ) -> Result<(Balance, Balance), RouterError> {
        if FactoryRef::get_pair(&self.data().factory, token_a, token_b).is_none() {
            FactoryRef::create_pair(&self.data().factory, token_a, token_b)?;
        };

        let (reserve_a, reserve_b) = get_reserves(
            self.data().factory.as_ref(),
            self.data().pair_code_hash.as_ref(),
            token_a,
            token_b,
        )?;
        if reserve_a == 0 && reserve_b == 0 {
            return Ok((amount_a_desired, amount_b_desired))
        }

        let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b)?;
        if amount_b_optimal <= amount_b_desired {
            ensure!(
                amount_b_optimal >= amount_b_min,
                RouterError::InsufficientBAmount
            );
            Ok((amount_a_desired, amount_b_optimal))
        } else {
            let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a)?;
            // amount_a_optimal <= amount_a_desired holds as amount_b_optimal > amount_b_desired
            ensure!(
                amount_a_optimal >= amount_a_min,
                RouterError::InsufficientAAmount
            );
            Ok((amount_a_optimal, amount_b_desired))
        }
    }

    fn _swap(
        &self,
        amounts: &Vec<Balance>,
        path: Vec<AccountId>,
        _to: AccountId,
    ) -> Result<(), RouterError> {
        let factory_ref = self.data().factory.as_ref();
        let pair_hash = self.data().pair_code_hash.as_ref();

        for i in 0..path.len() - 1 {
            let (input, output) = (path[i], path[i + 1]);
            let (token_0, _) = sort_tokens(input, output)?;
            let amount_out = amounts[i + 1];
            let (amount_0_out, amount_1_out) = if input == token_0 {
                (0, amount_out)
            } else {
                (amount_out, 0)
            };
            let to = if i < path.len() - 2 {
                pair_for(factory_ref, pair_hash, output, path[i + 2])?
            } else {
                _to
            };
            PairRef::swap_builder(
                &pair_for(factory_ref, pair_hash, input, output)?,
                amount_0_out,
                amount_1_out,
                to,
            )
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .fire()
            .unwrap()?
        }
        Ok(())
    }
}

#[modifier_definition]
pub fn ensure<T, F, R, E>(instance: &mut T, body: F, deadline: u64) -> Result<R, E>
where
    T: Storage<data::Data>,
    F: FnOnce(&mut T) -> Result<R, E>,
    E: From<RouterError>,
{
    ensure!(deadline >= T::env().block_timestamp(), RouterError::Expired);
    body(instance)
}
