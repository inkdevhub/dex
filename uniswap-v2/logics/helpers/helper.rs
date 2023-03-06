use crate::{
    helpers::math::casted_mul,
    traits::{
        factory::FactoryRef,
        pair::PairRef,
    },
};
use ink::prelude::vec::Vec;
use openbrush::traits::{
    AccountId,
    AccountIdExt,
    Balance,
};

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
}

pub fn sort_tokens(
    token_a: AccountId,
    token_b: AccountId,
) -> Result<(AccountId, AccountId), HelperError> {
    ensure!(token_a != token_b, HelperError::IdenticalAddresses);

    let (token_0, token_1) = if token_a < token_b {
        (token_a, token_b)
    } else {
        (token_b, token_a)
    };

    ensure!(!token_0.is_zero(), HelperError::ZeroAddress);

    Ok((token_0, token_1))
}

#[inline]
pub fn pair_for_on_chain(
    factory: &AccountId,
    token_a: AccountId,
    token_b: AccountId,
) -> Option<AccountId> {
    FactoryRef::get_pair(factory, token_a, token_b)
}

pub fn get_reserves(
    factory: &AccountId,
    token_a: AccountId,
    token_b: AccountId,
) -> Result<(Balance, Balance), HelperError> {
    let (token_0, _) = sort_tokens(token_a, token_b)?;
    let pair_contract =
        pair_for_on_chain(factory, token_a, token_b).ok_or(HelperError::PairNotFound)?;
    let (reserve_0, reserve_1, _) = PairRef::get_reserves(&pair_contract);
    if token_a == token_0 {
        Ok((reserve_0, reserve_1))
    } else {
        Ok((reserve_1, reserve_0))
    }
}

pub fn quote(
    amount_a: Balance,
    reserve_a: Balance,
    reserve_b: Balance,
) -> Result<Balance, HelperError> {
    ensure!(amount_a > 0, HelperError::InsufficientAmount);
    ensure!(
        reserve_a > 0 && reserve_b > 0,
        HelperError::InsufficientLiquidity
    );

    let amount_b: Balance = casted_mul(amount_a, reserve_b)
        .checked_div(reserve_a.into())
        .ok_or(HelperError::DivByZero)?
        .try_into()
        .map_err(|_| HelperError::CastOverflow)?;

    Ok(amount_b)
}

pub fn get_amount_out(
    amount_in: Balance,
    reserve_in: Balance,
    reserve_out: Balance,
) -> Result<Balance, HelperError> {
    ensure!(amount_in > 0, HelperError::InsufficientAmount);
    ensure!(
        reserve_in > 0 && reserve_out > 0,
        HelperError::InsufficientLiquidity
    );

    let amount_in_with_fee = casted_mul(amount_in, 997);

    let numerator = amount_in_with_fee
        .checked_mul(reserve_out.into())
        .ok_or(HelperError::MulOverFlow)?;

    let denominator = casted_mul(reserve_in, 1000)
        .checked_add(amount_in_with_fee)
        .ok_or(HelperError::AddOverFlow)?;

    let amount_out: Balance = numerator
        .checked_div(denominator)
        .ok_or(HelperError::DivByZero2)?
        .try_into()
        .map_err(|_| HelperError::CastOverflow2)?;

    Ok(amount_out)
}

pub fn get_amount_in(
    amount_out: Balance,
    reserve_in: Balance,
    reserve_out: Balance,
) -> Result<Balance, HelperError> {
    ensure!(amount_out > 0, HelperError::InsufficientAmount);
    ensure!(
        reserve_in > 0 && reserve_out > 0,
        HelperError::InsufficientLiquidity
    );

    let numerator = casted_mul(reserve_in, amount_out)
        .checked_mul(1000.into())
        .ok_or(HelperError::MulOverFlow)?;

    let denominator = casted_mul(
        reserve_out
            .checked_sub(amount_out)
            .ok_or(HelperError::SubUnderFlow)?,
        997,
    );

    let amount_in: Balance = numerator
        .checked_div(denominator)
        .ok_or(HelperError::DivByZero)?
        .checked_add(1.into())
        .ok_or(HelperError::AddOverFlow)?
        .try_into()
        .map_err(|_| HelperError::CastOverflow)?;

    Ok(amount_in)
}

pub fn get_amounts_out(
    factory: &AccountId,
    amount_in: Balance,
    path: &Vec<AccountId>,
) -> Result<Vec<Balance>, HelperError> {
    ensure!(path.len() >= 2, HelperError::InvalidPath);

    let mut amounts = Vec::with_capacity(path.len());
    amounts.push(amount_in);
    for i in 0..path.len() - 1 {
        let (reserve_in, reserve_out) = get_reserves(factory, path[i], path[i + 1])?;
        amounts.push(get_amount_out(amounts[i], reserve_in, reserve_out)?);
    }

    Ok(amounts)
}

pub fn get_amounts_in(
    factory: &AccountId,
    amount_out: Balance,
    path: &Vec<AccountId>,
) -> Result<Vec<Balance>, HelperError> {
    ensure!(path.len() >= 2, HelperError::InvalidPath);

    let mut amounts = Vec::with_capacity(path.len());
    unsafe {
        amounts.set_len(path.len());
    }
    amounts[path.len() - 1] = amount_out;
    for i in (0..path.len() - 1).rev() {
        let (reserve_in, reserve_out) = get_reserves(factory, path[i], path[i + 1])?;
        amounts[i] = get_amount_in(amounts[i + 1], reserve_in, reserve_out)?;
    }

    Ok(amounts)
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HelperError {
    IdenticalAddresses,
    ZeroAddress,
    InsufficientAmount,
    InsufficientLiquidity,
    DivByZero,
    CastOverflow,
    MulOverFlow,
    AddOverFlow,
    DivByZero2,
    CastOverflow2,
    InvalidPath,
    SubUnderFlow,
    PairNotFound,
}
