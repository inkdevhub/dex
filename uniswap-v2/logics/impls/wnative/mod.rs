pub use crate::traits::wnative::*;
use openbrush::{
    contracts::psp22::{
        self,
        PSP22Error,
    },
    traits::{
        Balance,
        Storage,
        String,
    },
};

impl<T: Storage<psp22::Data> + psp22::Internal> Wnative for T {
    default fn deposit(&mut self) -> Result<(), PSP22Error> {
        let transfer_value = Self::env().transferred_value();
        let caller = Self::env().caller();
        self._mint_to(caller, transfer_value)
    }

    default fn withdraw(&mut self, amount: Balance) -> Result<(), PSP22Error> {
        let caller = Self::env().caller();
        self._burn_from(caller, amount)?;
        Self::env()
            .transfer(caller, amount)
            .map_err(|_| PSP22Error::Custom(String::from("WNATIVE: transfer failed")))
    }
}
