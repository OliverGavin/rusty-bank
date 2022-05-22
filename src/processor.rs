use std::collections::HashMap;

use anyhow::Result;

use crate::{
    AccountStore, AccountWriter, Chargeback, Deposit, Dispute, Resolve, Transaction, TransactionId,
    TransactionReader, Withdrawal,
};

/// Indicates if a dispute is open or closed.
#[derive(Debug)]
enum DisputeStatus {
    Open,
    Closed,
}

/// Represents a dispute case
#[derive(Debug)]
struct DisputeCase {
    detail: Dispute,
    status: DisputeStatus,
}

impl DisputeCase {
    fn new(detail: Dispute) -> Self {
        DisputeCase {
            detail,
            status: DisputeStatus::Open,
        }
    }

    fn close(&mut self) {
        self.status = DisputeStatus::Closed;
    }
}

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
    store: S,
    deposits: HashMap<TransactionId, Deposit>,
    disputes: HashMap<TransactionId, DisputeCase>,
}

impl<S: AccountStore> TransactionProcessor<S> {
    /// Construct a new [`TransactionProcessor`].
    ///
    /// ### Parameters
    /// - store: The data store implementation.
    ///
    pub fn new(store: S) -> Self {
        TransactionProcessor {
            store,
            deposits: HashMap::new(),
            disputes: HashMap::new(),
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
                Ok(record) => match record.into() {
                    Ok(tx) => self.process_transaction(tx),
                    Err(err) => log::error!("Malformed transaction: {}", err),
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
        if let Err(err) = self.store.add_funds(deposit.client, deposit.amount) {
            log::info!("Cannot process {:?}: {}", deposit, err);
            return;
        };

        self.deposits.insert(deposit.tx, deposit);
    }

    fn process_withdrawal(&mut self, withdrawal: Withdrawal) {
        log::debug!("Processing withdrawal for {:?}", withdrawal);
        if let Err(err) = self
            .store
            .remove_funds(withdrawal.client, withdrawal.amount)
        {
            log::info!("Cannot process {:?}: {}", withdrawal, err)
        };
    }

    fn process_dispute(&mut self, dispute: Dispute) {
        log::debug!("Processing dispute for {:?}", dispute);

        let deposit = match self.deposits.get(&dispute.tx) {
            Some(deposit) => deposit,
            None => {
                log::info!(
                    "Cannot process dispute. No such transaction found for {:?}",
                    dispute
                );
                return;
            }
        };

        if deposit.client != dispute.client {
            log::info!(
                "Cannot process dispute. Client ID does not match for {:?} and {:?}",
                dispute,
                deposit
            );
            return;
        }

        if let Some(case) = self.disputes.get(&dispute.tx) {
            log::info!("Cannot process dispute. A case already exists {:?}", case);
            return;
        }

        if let Err(err) = self.store.hold_funds(dispute.client, deposit.amount) {
            log::info!("Cannot process {:?}: {}", dispute, err);
            return;
        };

        self.disputes.insert(dispute.tx, DisputeCase::new(dispute));
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

    use hamcrest2::assert_that;
    use hamcrest2::matches_regex;
    use hamcrest2::HamcrestMatcher;
    use itertools::Itertools;
    use log::Level;
    use mockall::predicate::eq;
    use mockall_double::double;
    use rust_decimal_macros::dec;

    use crate::Account;
    use crate::ClientId;
    use crate::TransactionId;
    use crate::TransactionRecord;
    use crate::TransactionType;

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
            let transactions = vec![TransactionRecord::new(
                TransactionType::Deposit,
                ClientId(1),
                TransactionId(1),
                Some(10.into()),
            )]
            .into_iter()
            .map(Ok);
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store
            .expect_add_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(10)))
            .returning(|_, _| Ok(()));

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
    }

    #[test]
    fn test_process_withdrawal_updates_store() {
        let mut reader = MockTransactionReader::new();
        reader.expect_read().returning(|| {
            let transactions = vec![TransactionRecord::new(
                TransactionType::Withdrawal,
                ClientId(1),
                TransactionId(2),
                Some(5.into()),
            )]
            .into_iter()
            .map(Ok);
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store
            .expect_remove_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(5)))
            .returning(|_, _| Ok(()));

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
    }

    #[test]
    fn test_process_dispute_updates_store() {
        let mut reader = MockTransactionReader::new();
        reader.expect_read().returning(|| {
            let transactions = vec![
                TransactionRecord::new(
                    TransactionType::Deposit,
                    ClientId(1),
                    TransactionId(1),
                    Some(10.into()),
                ),
                TransactionRecord::new(
                    TransactionType::Dispute,
                    ClientId(1),
                    TransactionId(1),
                    None,
                ),
            ]
            .into_iter()
            .map(Ok);
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store
            .expect_add_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(10)))
            .returning(|_, _| Ok(()));
        store
            .expect_hold_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(10)))
            .returning(|_, _| Ok(()));

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);
    }

    #[test]
    fn test_process_dispute_when_invalid_transaction_does_not_update_store() {
        testing_logger::setup();

        let mut reader = MockTransactionReader::new();
        reader.expect_read().returning(|| {
            let transactions = vec![
                // Err: No such transaction found
                TransactionRecord::new(
                    TransactionType::Dispute,
                    ClientId(1),
                    TransactionId(1),
                    None,
                ),
                // Ok
                TransactionRecord::new(
                    TransactionType::Deposit,
                    ClientId(1),
                    TransactionId(1),
                    Some(dec!(50)),
                ),
                // Err: Client ID does not match
                TransactionRecord::new(
                    TransactionType::Dispute,
                    ClientId(5),
                    TransactionId(1),
                    None,
                ),
                // Ok
                TransactionRecord::new(
                    TransactionType::Dispute,
                    ClientId(1),
                    TransactionId(1),
                    None,
                ),
                // Err: A case already exists
                TransactionRecord::new(
                    TransactionType::Dispute,
                    ClientId(1),
                    TransactionId(1),
                    None,
                ),
            ]
            .into_iter()
            .map(Ok);
            Box::new(transactions)
        });

        let mut store = MockAccountStore::new();
        store
            .expect_add_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(50)))
            .returning(|_, _| Ok(()));
        store
            .expect_hold_funds()
            .once()
            .with(eq(ClientId(1)), eq(dec!(50)))
            .returning(|_, _| Ok(()));

        let mut processor = TransactionProcessor::new(store);
        processor.process(reader);

        testing_logger::validate(|captured_logs| {
            let captured_logs = captured_logs
                .iter()
                .filter(|log| log.level <= Level::Info)
                .collect_vec();
            assert_eq!(captured_logs.len(), 3);
            assert_that!(
                captured_logs[0].body.to_owned(),
                matches_regex("No such transaction found")
            );
            assert_that!(
                captured_logs[1].body.to_owned(),
                matches_regex("Client ID does not match")
            );
            assert_that!(
                captured_logs[2].body.to_owned(),
                matches_regex("A case already exists")
            );
        });
    }

    #[test]
    fn test_export_writes_accounts_from_store() -> Result<()> {
        let mut store = MockAccountStore::new();
        store.expect_export().returning(|| {
            let accounts = vec![
                Account::empty(ClientId(1)),
                Account::empty(ClientId(2)),
                Account::empty(ClientId(3)),
            ]
            .into_iter();
            Box::new(accounts)
        });

        let mut writer = MockAccountWriter::new();
        writer.expect_write().times(3).returning(|_| Ok(()));

        let processor = TransactionProcessor::new(store);
        processor.export(writer)
    }
}
