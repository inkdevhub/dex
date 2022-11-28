use crate::traits::wnative::WnativeRef;
use ink_env::DefaultEnvironment;
use ink_prelude::vec::Vec;
use openbrush::{
    contracts::psp22::{
        PSP22Error,
        PSP22Ref,
    },
    traits::{
        AccountId,
        Balance,
    },
};

#[inline]
pub fn safe_transfer(token: AccountId, to: AccountId, value: Balance) -> Result<(), PSP22Error> {
    PSP22Ref::transfer(&token, to, value, Vec::new())
}

pub fn safe_transfer_native(to: AccountId, value: Balance) -> Result<(), TransferHelperError> {
    ink_env::transfer::<DefaultEnvironment>(to, value)
        .map_err(|_| TransferHelperError::TransferFailed)
}

#[inline]
pub fn safe_transfer_from(
    token: AccountId,
    from: AccountId,
    to: AccountId,
    value: Balance,
) -> Result<(), PSP22Error> {
    PSP22Ref::transfer_from(&token, from, to, value, Vec::new())
}

#[inline]
pub fn wrap(wnative: &AccountId, value: Balance) -> Result<(), PSP22Error> {
    WnativeRef::deposit_builder(wnative)
        .transferred_value(value)
        .fire()
        .unwrap()
}

#[inline]
pub fn unwrap(wnative: &AccountId, value: Balance) -> Result<(), PSP22Error> {
    WnativeRef::withdraw(wnative, value)
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TransferHelperError {
    TransferFailed,
}
