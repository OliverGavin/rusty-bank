use anyhow::Result;

use crate::{
    TransactionReader, AccountStore, AccountWriter, Transaction, Deposit, Withdrawal, Dispute, Resolve, Chargeback,
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
                Ok(record) => {
                    match record.into() {
                        Ok(tx) => self.process_transaction(tx),
                        Err(err) => log::error!("Malformed transaction: {}", err),
                    }
                },
                Err(err) => log::error!("Could not read transaction record: {}", err),
            }
        }
    }

    fn process_transaction(&mut self, transaction: Transaction) {
        match transaction {
            Transaction::Deposit(tx) => self.process_deposit(tx),
            Transaction::Withdrawal(tx) => self.process_withdrawal(tx),
            Transaction::Dispute(tx) => self.process_dispute(tx),
            Transaction::Resolve(tx) => self.process_resolve(tx),
            Transaction::Chargeback(tx) => self.process_chargeback(tx),
        }
    }

    fn process_deposit(&mut self, deposit: Deposit) {
        log::debug!("Processing deposit for {:?}", deposit);
        self.store.add_funds(deposit.client, deposit.amount);
    }

    fn process_withdrawal(&mut self, withdrawal: Withdrawal) {
        log::debug!("Processing withdrawal for {:?}", withdrawal);
        self.store.remove_funds(withdrawal.client, withdrawal.amount);
    }

    fn process_dispute(&self, dispute: Dispute) {
        log::debug!("Processing dispute for {:?}", dispute);
        todo!()
    }

    fn process_resolve(&self, resolve: Resolve) {
        log::debug!("Processing resolve for {:?}", resolve);
        todo!()
    }

    fn process_chargeback(&self, chargeback: Chargeback) {
        log::debug!("Processing chargeback for {:?}", chargeback);
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

    use crate::TransactionRecord;
    use crate::TransactionType;
    use crate::Account;
    use crate::ClientId;
    use crate::TransactionId;

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
                    TransactionRecord::new(
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
                    TransactionRecord::new(
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
