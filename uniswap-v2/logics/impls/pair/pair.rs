use crate::{
    ensure,
    helpers::{
        math::casted_mul,
        transfer_helper::safe_transfer,
    },
    traits::{
        factory::FactoryRef,
        types::WrappedU256,
    },
};
pub use crate::{
    impls::pair::*,
    traits::pair::*,
};
use openbrush::{
    contracts::{
        ownable::*,
        psp22::*,
        reentrancy_guard::*,
        traits::psp22::PSP22Ref,
    },
    modifiers,
    traits::{
        AccountId,
        AccountIdExt,
        Balance,
        Storage,
        Timestamp,
        ZERO_ADDRESS,
    },
};
use primitive_types::U256;
use sp_arithmetic::{
    traits::IntegerSquareRoot,
    FixedPointNumber,
    FixedU128,
};

pub const MINIMUM_LIQUIDITY: u128 = 1000;

pub trait Internal {
    fn _mint_fee(&mut self, reserve_0: Balance, reserve_1: Balance) -> Result<bool, PairError>;

    fn _update(
        &mut self,
        balance_0: Balance,
        balance_1: Balance,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<(), PairError>;

    fn _emit_mint_event(&self, _sender: AccountId, _amount_0: Balance, _amount_1: Balance);
    fn _emit_burn_event(
        &self,
        _sender: AccountId,
        _amount_0: Balance,
        _amount_1: Balance,
        _to: AccountId,
    );
    fn _emit_swap_event(
        &self,
        _sender: AccountId,
        _amount_0_in: Balance,
        _amount_1_in: Balance,
        _amount_0_out: Balance,
        _amount_1_out: Balance,
        _to: AccountId,
    );
    fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance);
}

impl<
        T: Storage<data::Data>
            + Storage<ownable::Data>
            + Storage<psp22::Data>
            + Storage<reentrancy_guard::Data>,
    > Pair for T
{
    default fn get_reserves(&self) -> (Balance, Balance, Timestamp) {
        (
            self.data::<data::Data>().reserve_0,
            self.data::<data::Data>().reserve_1,
            self.data::<data::Data>().block_timestamp_last,
        )
    }
    default fn price_0_cumulative_last(&self) -> WrappedU256 {
        self.data::<data::Data>().price_0_cumulative_last
    }

    default fn price_1_cumulative_last(&self) -> WrappedU256 {
        self.data::<data::Data>().price_1_cumulative_last
    }

    #[modifiers(only_owner)]
    default fn initialize(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
    ) -> Result<(), PairError> {
        self.data::<data::Data>().token_0 = token_0;
        self.data::<data::Data>().token_1 = token_1;
        Ok(())
    }

    #[modifiers(non_reentrant)]
    default fn mint(&mut self, to: AccountId) -> Result<Balance, PairError> {
        let reserves = self.get_reserves();
        let contract = Self::env().account_id();
        let balance_0 = PSP22Ref::balance_of(&self.data::<data::Data>().token_0, contract);
        let balance_1 = PSP22Ref::balance_of(&self.data::<data::Data>().token_1, contract);
        let amount_0 = balance_0
            .checked_sub(reserves.0)
            .ok_or(PairError::SubUnderFlow1)?;
        let amount_1 = balance_1
            .checked_sub(reserves.1)
            .ok_or(PairError::SubUnderFlow2)?;

        let fee_on = self._mint_fee(reserves.0, reserves.1)?;
        let total_supply = self.data::<psp22::Data>().supply;

        let liquidity;
        if total_supply == 0 {
            let liq = amount_0
                .checked_mul(amount_1)
                .ok_or(PairError::MulOverFlow1)?;
            liquidity = liq
                .integer_sqrt()
                .checked_sub(MINIMUM_LIQUIDITY)
                .ok_or(PairError::SubUnderFlow3)?;
            self._mint_to(ZERO_ADDRESS.into(), MINIMUM_LIQUIDITY)?;
        } else {
            let liquidity_1 = amount_0
                .checked_mul(total_supply)
                .ok_or(PairError::MulOverFlow2)?
                .checked_div(reserves.0)
                .ok_or(PairError::DivByZero1)?;
            let liquidity_2 = amount_1
                .checked_mul(total_supply)
                .ok_or(PairError::MulOverFlow3)?
                .checked_div(reserves.1)
                .ok_or(PairError::DivByZero2)?;
            liquidity = min(liquidity_1, liquidity_2);
        }

        ensure!(liquidity > 0, PairError::InsufficientLiquidityMinted);

        self._mint_to(to, liquidity)?;

        self._update(balance_0, balance_1, reserves.0, reserves.1)?;

        if fee_on {
            self.data::<data::Data>().k_last = casted_mul(reserves.0, reserves.1).into();
        }

        self._emit_mint_event(Self::env().caller(), amount_0, amount_1);

        Ok(liquidity)
    }

    #[modifiers(non_reentrant)]
    default fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), PairError> {
        let reserves = self.get_reserves();
        let contract = Self::env().account_id();
        let token_0 = self.data::<data::Data>().token_0;
        let token_1 = self.data::<data::Data>().token_1;
        let mut balance_0 = PSP22Ref::balance_of(&token_0, contract);
        let mut balance_1 = PSP22Ref::balance_of(&token_1, contract);
        let liquidity = self._balance_of(&contract);

        let fee_on = self._mint_fee(reserves.0, reserves.1)?;
        let total_supply = self.data::<psp22::Data>().supply;
        let amount_0 = liquidity
            .checked_mul(balance_0)
            .ok_or(PairError::MulOverFlow5)?
            .checked_div(total_supply)
            .ok_or(PairError::DivByZero3)?;
        let amount_1 = liquidity
            .checked_mul(balance_1)
            .ok_or(PairError::MulOverFlow6)?
            .checked_div(total_supply)
            .ok_or(PairError::DivByZero4)?;

        ensure!(
            amount_0 > 0 && amount_1 > 0,
            PairError::InsufficientLiquidityBurned
        );

        self._burn_from(contract, liquidity)?;

        safe_transfer(token_0, to, amount_0)?;
        safe_transfer(token_1, to, amount_1)?;

        balance_0 = PSP22Ref::balance_of(&token_0, contract);
        balance_1 = PSP22Ref::balance_of(&token_1, contract);

        self._update(balance_0, balance_1, reserves.0, reserves.1)?;

        if fee_on {
            self.data::<data::Data>().k_last = casted_mul(reserves.0, reserves.1).into();
        }

        self._emit_burn_event(Self::env().caller(), amount_0, amount_1, to);

        Ok((amount_0, amount_1))
    }

    #[modifiers(non_reentrant)]
    default fn swap(
        &mut self,
        amount_0_out: Balance,
        amount_1_out: Balance,
        to: AccountId,
    ) -> Result<(), PairError> {
        ensure!(
            amount_0_out > 0 || amount_1_out > 0,
            PairError::InsufficientOutputAmount
        );
        let reserves = self.get_reserves();
        ensure!(
            amount_0_out < reserves.0 && amount_1_out < reserves.1,
            PairError::InsufficientLiquidity
        );

        let token_0 = self.data::<data::Data>().token_0;
        let token_1 = self.data::<data::Data>().token_1;

        ensure!(to != token_0 && to != token_1, PairError::InvalidTo);
        if amount_0_out > 0 {
            safe_transfer(token_0, to, amount_0_out)?;
        }
        if amount_1_out > 0 {
            safe_transfer(token_1, to, amount_1_out)?;
        }
        let contract = Self::env().account_id();
        let balance_0 = PSP22Ref::balance_of(&token_0, contract);
        let balance_1 = PSP22Ref::balance_of(&token_1, contract);

        let amount_0_in = if balance_0
            > reserves
                .0
                .checked_sub(amount_0_out)
                .ok_or(PairError::SubUnderFlow4)?
        {
            balance_0
                .checked_sub(
                    reserves
                        .0
                        .checked_sub(amount_0_out)
                        .ok_or(PairError::SubUnderFlow5)?,
                )
                .ok_or(PairError::SubUnderFlow6)?
        } else {
            0
        };
        let amount_1_in = if balance_1
            > reserves
                .1
                .checked_sub(amount_1_out)
                .ok_or(PairError::SubUnderFlow7)?
        {
            balance_1
                .checked_sub(
                    reserves
                        .1
                        .checked_sub(amount_1_out)
                        .ok_or(PairError::SubUnderFlow8)?,
                )
                .ok_or(PairError::SubUnderFlow9)?
        } else {
            0
        };

        ensure!(
            amount_0_in > 0 || amount_1_in > 0,
            PairError::InsufficientInputAmount
        );

        let balance_0_adjusted = balance_0
            .checked_mul(1000)
            .ok_or(PairError::MulOverFlow7)?
            .checked_sub(amount_0_in.checked_mul(3).ok_or(PairError::MulOverFlow8)?)
            .ok_or(PairError::SubUnderFlow10)?;
        let balance_1_adjusted = balance_1
            .checked_mul(1000)
            .ok_or(PairError::MulOverFlow9)?
            .checked_sub(amount_1_in.checked_mul(3).ok_or(PairError::MulOverFlow10)?)
            .ok_or(PairError::SubUnderFlow11)?;

        // Cast to U256 to prevent Overflow
        ensure!(
            casted_mul(balance_0_adjusted, balance_1_adjusted)
                >= casted_mul(reserves.0, reserves.1)
                    .checked_mul(1000u128.pow(2).into())
                    .ok_or(PairError::MulOverFlow14)?,
            PairError::K
        );

        self._update(balance_0, balance_1, reserves.0, reserves.1)?;

        self._emit_swap_event(
            Self::env().caller(),
            amount_0_in,
            amount_1_in,
            amount_0_out,
            amount_1_out,
            to,
        );
        Ok(())
    }

    #[modifiers(non_reentrant)]
    default fn skim(&mut self, to: AccountId) -> Result<(), PairError> {
        let contract = Self::env().account_id();
        let reserve_0 = self.data::<data::Data>().reserve_0;
        let reserve_1 = self.data::<data::Data>().reserve_1;
        let token_0 = self.data::<data::Data>().token_0;
        let token_1 = self.data::<data::Data>().token_1;
        let balance_0 = PSP22Ref::balance_of(&token_0, contract);
        let balance_1 = PSP22Ref::balance_of(&token_1, contract);
        safe_transfer(
            token_0,
            to,
            balance_0
                .checked_sub(reserve_0)
                .ok_or(PairError::SubUnderFlow12)?,
        )?;
        safe_transfer(
            token_1,
            to,
            balance_1
                .checked_sub(reserve_1)
                .ok_or(PairError::SubUnderFlow13)?,
        )?;
        Ok(())
    }

    #[modifiers(non_reentrant)]
    default fn sync(&mut self) -> Result<(), PairError> {
        let contract = Self::env().account_id();
        let reserve_0 = self.data::<data::Data>().reserve_0;
        let reserve_1 = self.data::<data::Data>().reserve_1;
        let token_0 = self.data::<data::Data>().token_0;
        let token_1 = self.data::<data::Data>().token_1;
        let balance_0 = PSP22Ref::balance_of(&token_0, contract);
        let balance_1 = PSP22Ref::balance_of(&token_1, contract);
        self._update(balance_0, balance_1, reserve_0, reserve_1)
    }

    default fn get_token_0(&self) -> AccountId {
        self.data::<data::Data>().token_0
    }

    default fn get_token_1(&self) -> AccountId {
        self.data::<data::Data>().token_1
    }
}

fn min(x: u128, y: u128) -> u128 {
    if x < y {
        return x
    }
    y
}

#[inline]
fn update_cumulative(
    price_0_cumulative_last: WrappedU256,
    price_1_cumulative_last: WrappedU256,
    time_elapsed: U256,
    reserve_0: Balance,
    reserve_1: Balance,
) -> (WrappedU256, WrappedU256) {
    let price_cumulative_last_0: WrappedU256 = U256::from(
        FixedU128::checked_from_rational(reserve_1, reserve_0)
            .unwrap_or_default()
            .into_inner(),
    )
    .saturating_mul(time_elapsed)
    .saturating_add(price_0_cumulative_last.into())
    .into();
    let price_cumulative_last_1: WrappedU256 = U256::from(
        FixedU128::checked_from_rational(reserve_0, reserve_1)
            .unwrap_or_default()
            .into_inner(),
    )
    .saturating_mul(time_elapsed)
    .saturating_add(price_1_cumulative_last.into())
    .into();
    (price_cumulative_last_0, price_cumulative_last_1)
}

impl<T: Storage<data::Data> + Storage<psp22::Data>> Internal for T {
    default fn _mint_fee(
        &mut self,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<bool, PairError> {
        let fee_to = FactoryRef::fee_to(&self.data::<data::Data>().factory);
        let fee_on = !fee_to.is_zero();
        let k_last: U256 = self.data::<data::Data>().k_last.into();
        if fee_on {
            if !k_last.is_zero() {
                let root_k: Balance = casted_mul(reserve_0, reserve_1)
                    .integer_sqrt()
                    .try_into()
                    .map_err(|_| PairError::CastOverflow1)?;
                let root_k_last = k_last
                    .integer_sqrt()
                    .try_into()
                    .map_err(|_| PairError::CastOverflow2)?;
                if root_k > root_k_last {
                    let total_supply = self.data::<psp22::Data>().supply;
                    let numerator = total_supply
                        .checked_mul(
                            root_k
                                .checked_sub(root_k_last)
                                .ok_or(PairError::SubUnderFlow14)?,
                        )
                        .ok_or(PairError::MulOverFlow13)?;
                    let denominator = root_k
                        .checked_mul(5)
                        .ok_or(PairError::MulOverFlow13)?
                        .checked_add(root_k_last)
                        .ok_or(PairError::AddOverflow1)?;
                    let liquidity = numerator
                        .checked_div(denominator)
                        .ok_or(PairError::DivByZero5)?;
                    if liquidity > 0 {
                        self._mint_to(fee_to, liquidity)?;
                    }
                }
            }
        } else if !k_last.is_zero() {
            self.data::<data::Data>().k_last = 0.into();
        }
        Ok(fee_on)
    }

    default fn _update(
        &mut self,
        balance_0: Balance,
        balance_1: Balance,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<(), PairError> {
        ensure!(
            balance_0 <= u128::MAX && balance_1 <= u128::MAX,
            PairError::Overflow
        );
        let now = Self::env().block_timestamp();
        let last_timestamp = self.data::<data::Data>().block_timestamp_last;
        if now != last_timestamp {
            let (price_0_cumulative_last, price_1_cumulative_last) = update_cumulative(
                self.data::<data::Data>().price_0_cumulative_last,
                self.data::<data::Data>().price_1_cumulative_last,
                now.saturating_sub(last_timestamp).into(),
                reserve_0,
                reserve_1,
            );
            self.data::<data::Data>().price_0_cumulative_last = price_0_cumulative_last;
            self.data::<data::Data>().price_1_cumulative_last = price_1_cumulative_last;
        }
        self.data::<data::Data>().reserve_0 = balance_0;
        self.data::<data::Data>().reserve_1 = balance_1;
        self.data::<data::Data>().block_timestamp_last = now;

        self._emit_sync_event(reserve_0, reserve_1);
        Ok(())
    }

    default fn _emit_mint_event(&self, _sender: AccountId, _amount_0: Balance, _amount_1: Balance) {
    }
    default fn _emit_burn_event(
        &self,
        _sender: AccountId,
        _amount_0: Balance,
        _amount_1: Balance,
        _to: AccountId,
    ) {
    }
    default fn _emit_swap_event(
        &self,
        _sender: AccountId,
        _amount_0_in: Balance,
        _amount_1_in: Balance,
        _amount_0_out: Balance,
        _amount_1_out: Balance,
        _to: AccountId,
    ) {
    }
    default fn _emit_sync_event(&self, _reserve_0: Balance, _reserve_1: Balance) {}
}

#[cfg(test)]
mod tests {
    use primitive_types::U256;
    use sp_arithmetic::FixedU128;

    use super::update_cumulative;

    #[test]
    fn update_cumulative_from_zero_time_elapsed() {
        let (cumulative0, cumulative1) = update_cumulative(0.into(), 0.into(), 0.into(), 10, 10);
        assert_eq!(cumulative0, 0.into());
        assert_eq!(cumulative1, 0.into());
    }

    #[test]
    fn update_cumulative_from_one_time_elapsed() {
        let (cumulative0, cumulative1) = update_cumulative(0.into(), 0.into(), 1.into(), 10, 10);
        assert_eq!(
            FixedU128::from_inner(U256::from(cumulative0).as_u128()),
            1.into()
        );
        assert_eq!(
            FixedU128::from_inner(U256::from(cumulative1).as_u128()),
            1.into()
        );
    }
}
