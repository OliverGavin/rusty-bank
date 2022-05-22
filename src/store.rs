use std::collections::HashMap;

use anyhow::{Result, Error};
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
        Account{
            client,
            held: 0.into(),
            total: 0.into(),
            locked: false
        }
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
    accounts: HashMap<ClientId, Account>
}

impl InMemoryAccountStore {
    /// Construct a new [`InMemoryAccountStore`].
    pub fn new() -> Self {
        InMemoryAccountStore {
            accounts: HashMap::new()
        }
    }

    fn get_account(&mut self, client: ClientId) -> Result<&mut Account> {
        // self.accounts.mut
        let account = self.accounts.entry(client)
            .or_insert_with(|| Account::empty(client));
        match account.locked {
            true => Err(Error::msg(format!("Account is locked: {:?}", account))),
            false => Ok(account),
        }
    }
}

impl AccountStore for InMemoryAccountStore {
    fn add_funds(&mut self, client: ClientId, _amount: Decimal) -> Result<()> {
        let _account = self.get_account(client)?;
        // TODO
        Ok(())
    }

    fn remove_funds(&mut self, client: ClientId, _amount: Decimal) -> Result<()> {
        let _account = self.get_account(client)?;
        // TODO
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
}
