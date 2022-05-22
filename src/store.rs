use std::collections::HashMap;

use anyhow::{Error, Result};
use rust_decimal::Decimal;

use crate::ClientId;

/// Internal state of a client's account
#[derive(Debug)]
pub struct Account {
    pub client: ClientId,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    /// Create an empty account with a balance of zero
    pub fn empty(client: ClientId) -> Self {
        Account {
            client,
            held: 0.into(),
            total: 0.into(),
            locked: false,
        }
    }

    pub fn get_available(&self) -> Decimal {
        self.total - self.held
    }
}

/// A trait for any account store implementation.
#[cfg_attr(test, mockall::automock)]
pub trait AccountStore {
    /// Adds funds to a client's account.
    fn add_funds(&mut self, client: ClientId, amount: Decimal) -> Result<()>;

    /// Removes funds to a client's account.
    fn remove_funds(&mut self, client: ClientId, amount: Decimal) -> Result<()>;

    /// Holds funds to a client's account.
    fn hold_funds(&mut self, client: ClientId, amount: Decimal) -> Result<()>;

    /// Exports all accounts as an iterator, consuming the store.
    fn export(self) -> Box<dyn Iterator<Item = Account>>;
}

/// An in-memory implementation of [`AccountStore`].
pub struct InMemoryAccountStore {
    accounts: HashMap<ClientId, Account>,
}

impl InMemoryAccountStore {
    /// Construct a new [`InMemoryAccountStore`].
    pub fn new() -> Self {
        InMemoryAccountStore {
            accounts: HashMap::new(),
        }
    }

    fn get_account(&mut self, client: ClientId) -> Result<&mut Account> {
        // self.accounts.mut
        let account = self
            .accounts
            .entry(client)
            .or_insert_with(|| Account::empty(client));
        match account.locked {
            true => Err(Error::msg(format!("Account is locked: {:?}", account))),
            false => Ok(account),
        }
    }
}

impl AccountStore for InMemoryAccountStore {
    fn add_funds(&mut self, client: ClientId, amount: Decimal) -> Result<()> {
        let account = self.get_account(client)?;
        account.total += amount;
        Ok(())
    }

    fn remove_funds(&mut self, client: ClientId, amount: Decimal) -> Result<()> {
        let account = self.get_account(client)?;
        if amount > account.get_available() {
            return Err(Error::msg(format!(
                "Insufficient funds available to withdraw '{}' for {:?}",
                amount, account
            )));
        }
        account.total -= amount;
        Ok(())
    }

    fn hold_funds(&mut self, client: ClientId, _amount: Decimal) -> Result<()> {
        let _account = self.get_account(client)?;
        // TODO
        Ok(())
    }

    fn export(self) -> Box<dyn Iterator<Item = Account>> {
        Box::new(self.accounts.into_iter().map(|(_, account)| account))
    }
}

#[cfg(test)]
mod test {
    use rust_decimal_macros::dec;

    use super::*;

    #[test]
    fn test_get_account() {
        let mut store = InMemoryAccountStore::new();
        let result = store.get_account(ClientId(1));
        assert_eq!(true, result.is_ok());

        result.unwrap().locked = true;
        let result = store.get_account(ClientId(1));
        assert_eq!(true, result.is_err());
    }

    #[test]
    fn test_add_funds() -> Result<()> {
        let mut store = InMemoryAccountStore::new();
        store.add_funds(ClientId(2), dec!(20))?;
        store.add_funds(ClientId(2), dec!(5))?;

        let account = store.get_account(ClientId(2))?;
        assert_eq!(dec!(25), account.total);
        assert_eq!(dec!(0), account.held);

        Ok(())
    }

    #[test]
    fn test_remove_funds() -> Result<()> {
        let mut store = InMemoryAccountStore::new();
        store.add_funds(ClientId(2), dec!(20))?;
        store.remove_funds(ClientId(2), dec!(5))?;

        let account = store.get_account(ClientId(2))?;
        assert_eq!(dec!(15), account.total);
        assert_eq!(dec!(0), account.held);

        Ok(())
    }

    #[test]
    fn test_remove_funds_when_insufficient_available() -> Result<()> {
        let mut store = InMemoryAccountStore::new();
        store.add_funds(ClientId(2), dec!(20))?;
        assert_eq!(true, store.remove_funds(ClientId(2), dec!(100)).is_err());

        let account = store.get_account(ClientId(2))?;
        assert_eq!(dec!(20), account.total);
        assert_eq!(dec!(0), account.held);

        Ok(())
    }
}
