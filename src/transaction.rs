//! Serdes for transactions

use anyhow::{Context, Error, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{client::ClientId, TransactionRecord, TransactionType};

/// Represents a transaction ID as it's own type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct TransactionId(pub u32);

/// Internal transaction representation.
///
/// Each transaction variant is implemented as its own struct.
#[derive(Debug)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

#[derive(Debug)]
pub struct Deposit {
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Decimal,
}

#[derive(Debug)]
pub struct Withdrawal {
    pub client: ClientId,
    pub tx: TransactionId,
    pub amount: Decimal,
}

#[derive(Debug)]
pub struct Dispute {
    pub client: ClientId,
    pub tx: TransactionId,
}

#[derive(Debug)]
pub struct Resolve {
    pub client: ClientId,
    pub tx: TransactionId,
}

#[derive(Debug)]
pub struct Chargeback {
    pub client: ClientId,
    pub tx: TransactionId,
}

/// Supports conversion of a [`TransactionRecord`] to a [`Transaction`].
// Having to convert from the TransactionRecord serde to a Transaction is a bit verbose
// and is due to lacking features in rust-csv where internally-tagged enums are not supported.
// However, it does allow more optimal usage of the rust type system.
// Additionally it provides an opportunity for more advanced validations.
impl From<TransactionRecord> for Result<Transaction> {
    /// Converts a [`TransactionRecord`] to a [`Result<Transaction>`].
    /// An error is returned if validation fails or if expected fields are missing.
    fn from(record: TransactionRecord) -> Self {
        // validate the record fields
        if let Some(amount) = record.amount {
            // dispute, resolve and chargeback transactions should not have an amount
            if let TransactionType::Dispute
            | TransactionType::Resolve
            | TransactionType::Chargeback = record.transaction_type
            {
                return Err(Error::msg(format!(
                    "Unexpected amount field in {:?}",
                    &record
                )));
            }
            // amount must be a positive non-zero number
            if amount <= 0.into() {
                return Err(Error::msg(format!(
                    "Expected positive amount for {:?}",
                    &record
                )));
            }
        }

        // attempt to convert records to transactions
        match record.transaction_type {
            TransactionType::Deposit => Ok(Transaction::Deposit(Deposit {
                client: record.client,
                tx: record.tx,
                amount: record
                    .amount
                    .with_context(|| format!("Expected amount for {:?}", &record))?,
            })),
            TransactionType::Withdrawal => Ok(Transaction::Withdrawal(Withdrawal {
                client: record.client,
                tx: record.tx,
                amount: record
                    .amount
                    .with_context(|| format!("Expected amount for {:?}", &record))?,
            })),
            TransactionType::Dispute => Ok(Transaction::Dispute(Dispute {
                client: record.client,
                tx: record.tx,
            })),
            TransactionType::Resolve => Ok(Transaction::Resolve(Resolve {
                client: record.client,
                tx: record.tx,
            })),
            TransactionType::Chargeback => Ok(Transaction::Chargeback(Chargeback {
                client: record.client,
                tx: record.tx,
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use rust_decimal_macros::dec;
    use test_case::test_case;

    #[test_case(TransactionType::Deposit,    ClientId(1), TransactionId(1), Some(dec!(10)); "when deposit")]
    #[test_case(TransactionType::Withdrawal, ClientId(1), TransactionId(1), Some(dec!(10)); "when withdrawal")]
    #[test_case(TransactionType::Dispute,    ClientId(1), TransactionId(1), None;           "when dispute")]
    #[test_case(TransactionType::Resolve,    ClientId(1), TransactionId(1), None;           "when resolve")]
    #[test_case(TransactionType::Chargeback, ClientId(1), TransactionId(1), None;           "when chargeback")]
    fn test_from_when_valid_record(
        transaction_type: TransactionType,
        client: ClientId,
        tx: TransactionId,
        amount: Option<Decimal>,
    ) {
        let record = TransactionRecord::new(transaction_type, client, tx, amount);
        let result: Result<Transaction> = record.into();
        result.unwrap();
    }

    #[test_case(TransactionType::Deposit,    ClientId(1), TransactionId(1), Some(dec!(-10)); "when deposit and negative amount")]
    #[test_case(TransactionType::Withdrawal, ClientId(1), TransactionId(1), Some(dec!(-10)); "when withdrawal and negative amount")]
    #[test_case(TransactionType::Deposit,    ClientId(1), TransactionId(1), None;            "when deposit and missing amount")]
    #[test_case(TransactionType::Withdrawal, ClientId(1), TransactionId(1), None;            "when withdrawal and missing amount")]
    #[test_case(TransactionType::Dispute,    ClientId(1), TransactionId(1), Some(dec!(10));  "when dispute and some ammount")]
    #[test_case(TransactionType::Resolve,    ClientId(1), TransactionId(1), Some(dec!(10));  "when resolve and some ammount")]
    #[test_case(TransactionType::Chargeback, ClientId(1), TransactionId(1), Some(dec!(10));  "when chargeback and some ammount")]
    #[should_panic]
    fn test_from_when_invalid_record(
        transaction_type: TransactionType,
        client: ClientId,
        tx: TransactionId,
        amount: Option<Decimal>,
    ) {
        let record = TransactionRecord::new(transaction_type, client, tx, amount);
        let result: Result<Transaction> = record.into();
        result.unwrap();
    }
}
