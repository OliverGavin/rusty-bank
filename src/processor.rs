use anyhow::Result;
use rust_decimal::Decimal;

use crate::{
    ClientId, Transaction, TransactionReader, TransactionId,
    TransactionType, AccountStore, AccountWriter,
};

/// A transaction processor which implements the key operations on client accounts.
///
/// [`TransactionProcessor`] supports implementations of the [`AccountStore`], [`TransactionReader`]
/// and [`AccountWriter`] traits allowing changes in reading, storing and writing to be implemented
/// in isolation from the core transaction processing logic.
///
/// ### Generic Parameters
/// - [`<S: AccountStore>`](AccountStore): The data store type.
///
pub struct TransactionProcessor<S: AccountStore> {
    store: S
}

impl<S: AccountStore> TransactionProcessor<S> {
    /// Construct a new [`TransactionProcessor`].
    ///
    /// ### Parameters
    /// - store: The data store implementation.
    ///
    pub fn new(store: S) -> Self {
        TransactionProcessor {
            store
        }
    }

    /// Process transactions.
    ///
    /// Using a supplied reader, reads and processes each transaction and maintains client account state.
    ///
    /// ### Parameters
    /// - reader: The transaction reader.
    pub fn process(&mut self, mut reader: impl TransactionReader) {
        for result in reader.read() {
            match result {
                Ok(transaction) => self.process_transaction(transaction),
                Err(err) => log::error!("Could not read transaction: {}", err),
            }
        }
    }

    fn process_transaction(&mut self, transaction: Transaction) {
        let (transaction_type, client, tx, amount) = (
            transaction.transaction_type,
            transaction.client,
            transaction.tx,
            transaction.amount,
        );
        match transaction_type {
            TransactionType::Deposit => self.process_deposit(client, tx, amount.unwrap()),
            TransactionType::Withdrawal => self.process_withdrawal(client, tx, amount.unwrap()),
            TransactionType::Dispute => self.process_dispute(client, tx),
            TransactionType::Resolve => self.process_resolve(client, tx),
            TransactionType::Chargeback => self.process_chargeback(client, tx),
        }
    }

    fn process_deposit(&mut self, client: ClientId, tx: TransactionId, amount: Decimal) {
        log::debug!(
            "Processing deposit (client='{:?}', tx='{:?}', amount='{:?}')",
            client,
            tx,
            amount
        );
        self.store.add_funds(client, amount);
    }

    fn process_withdrawal(&mut self, client: ClientId, tx: TransactionId, amount: Decimal) {
        log::debug!(
            "Processing withdrawal (client='{:?}', tx='{:?}', amount='{:?}')",
            client,
            tx,
            amount
        );
        self.store.remove_funds(client, amount);
    }

    fn process_dispute(&self, client: ClientId, tx: TransactionId) {
        log::debug!(
            "Processing dispute (client='{:?}', tx='{:?}')",
            client,
            tx
        );
        todo!()
    }

    fn process_resolve(&self, client: ClientId, tx: TransactionId) {
        log::debug!(
            "Processing resolve (client='{:?}', tx='{:?}')",
            client,
            tx
        );
        todo!()
    }

    fn process_chargeback(&self, client: ClientId, tx: TransactionId) {
        log::debug!(
            "Processing chargeback (client='{:?}', tx='{:?}')",
            client,
            tx
        );
        todo!()
    }

    /// Export accounts processed.
    ///
    /// Using a supplied writer, writes each client account state.
    /// The writer is consumed to ensure it is dropped once this method completes,
    /// allowing for files to be flushed or other resources to be released.
    ///
    /// The [`TransactionProcessor`] is also consumed, preventing further transaction
    /// processing modifying the state of accounts already written.
    ///
    /// ### Parameters
    /// - writer: The implementation of the account writer.
    pub fn export(self, mut writer: impl AccountWriter) -> Result<()> {
        for account in self.store.export() {
            writer.write(&account.into())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use mockall::predicate::eq;
    use mockall_double::double;
    use rust_decimal_macros::dec;

    use crate::Account;
    #[double]
    use crate::AccountStore as MockAccountStore;
    #[double]
    use crate::AccountWriter as MockAccountWriter;
    #[double]
    use crate::TransactionReader as MockTransactionReader;

    #[test]
    fn test_process_deposit_updates_store() {
        let mut reader = MockTransactionReader::new();
        reader.expect_read().returning(|| {
            let transactions = vec![
                Ok(
                    Transaction::new(
                        TransactionType::Deposit,
                        ClientId(1),
                        TransactionId(1),
                        Some(10.into())
                    )
                ),
            ].into_iter();
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store.expect_add_funds()
             .times(1)
             .with(eq(ClientId(1)), eq(dec!(10)))
             .return_const(());

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
    }

    #[test]
    fn test_process_withdrawal_updates_store() {
        let mut reader = MockTransactionReader::new();
        reader.expect_read().returning(|| {
            let transactions = vec![
                Ok(
                    Transaction::new(
                        TransactionType::Withdrawal,
                        ClientId(1),
                        TransactionId(2),
                        Some(5.into())
                    )
                ),
            ].into_iter();
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store.expect_remove_funds()
             .times(1)
             .with(eq(ClientId(1)), eq(dec!(5)))
             .return_const(());

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
    }

    #[test]
    fn test_export_writes_accounts_from_store() -> Result<()> {
        let mut store = MockAccountStore::new();
        store.expect_export().returning(|| {
            let accounts = vec![
                Account::empty(ClientId(1)),
                Account::empty(ClientId(2)),
                Account::empty(ClientId(3)),
            ].into_iter();
            Box::new(accounts)
        });

        let mut writer = MockAccountWriter::new();
        writer.expect_write()
              .times(3)
              .returning(|_| Ok(()));

        let processor = TransactionProcessor::new(store);
        processor.export(writer)
    }
}
