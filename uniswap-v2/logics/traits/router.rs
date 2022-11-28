use super::{
    factory::FactoryError,
    pair::PairError,
};
use crate::helpers::{
    helper::HelperError,
    transfer_helper::TransferHelperError,
};
use ink_prelude::vec::Vec;
use openbrush::{
    contracts::psp22::PSP22Error,
    traits::{
        AccountId,
        Balance,
    },
};

#[openbrush::wrapper]
pub type RouterRef = dyn Router;

#[openbrush::trait_definition]
pub trait Router {
    #[ink(message)]
    fn factory(&self) -> AccountId;

    #[ink(message)]
    fn wnative(&self) -> AccountId;

    #[ink(message)]
    fn add_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance, Balance), RouterError>;

    #[ink(message)]
    fn remove_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        liquidity: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance), RouterError>;

    #[ink(message, payable)]
    fn add_liquidity_native(
        &mut self,
        token: AccountId,
        amount_token_desired: Balance,
        amount_token_min: Balance,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance, Balance), RouterError>;

    #[ink(message)]
    fn remove_liquidity_native(
        &mut self,
        token: AccountId,
        liquidity: Balance,
        amount_token_min: Balance,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(Balance, Balance), RouterError>;

    #[ink(message)]
    fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: Balance,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message)]
    fn swap_tokens_for_exact_tokens(
        &mut self,
        amount_out: Balance,
        amount_in_max: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message, payable)]
    fn swap_exact_native_for_tokens(
        &mut self,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message)]
    fn swap_tokens_for_exact_native(
        &mut self,
        amount_out: Balance,
        amount_in_max: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message)]
    fn swap_exact_tokens_for_native(
        &mut self,
        amount_in: Balance,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message, payable)]
    fn swap_native_for_exact_tokens(
        &mut self,
        amount_out: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message)]
    fn quote(
        &self,
        amount_a: Balance,
        reserve_a: Balance,
        reserve_b: Balance,
    ) -> Result<Balance, RouterError>;

    #[ink(message)]
    fn get_amount_out(
        &self,
        amount_in: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, RouterError>;

    #[ink(message)]
    fn get_amount_in(
        &self,
        amount_out: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, RouterError>;

    #[ink(message)]
    fn get_amounts_out(
        &self,
        amount_in: Balance,
        path: Vec<AccountId>,
    ) -> Result<Vec<Balance>, RouterError>;

    #[ink(message)]
    fn get_amounts_in(
        &self,
        amount_out: Balance,
        path: Vec<AccountId>,
    ) -> Result<Vec<Balance>, RouterError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RouterError {
    PSP22Error(PSP22Error),
    FactoryError(FactoryError),
    PairError(PairError),
    HelperError(HelperError),
    TransferHelperError(TransferHelperError),
    PairNotFound,
    InsufficientAmount,
    InsufficientAAmount,
    InsufficientOutputAmount,
    ExcessiveInputAmount,
    InsufficientBAmount,
    InsufficientLiquidity,
    ZeroAddress,
    IdenticalAddresses,
    Expired,
    SubUnderFlow,
    MulOverFlow,
    DivByZero,
    TransferFailed,
    InvalidPath,
}

macro_rules! impl_froms {
    ( $( $error:ident ),* ) => {
        $(
            impl From<$error> for RouterError {
                fn from(error: $error) -> Self {
                    RouterError::$error(error)
                }
            }
        )*
    };
}

impl_froms!(
    PSP22Error,
    FactoryError,
    PairError,
    HelperError,
    TransferHelperError
);
